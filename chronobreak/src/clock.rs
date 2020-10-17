use crate::mock::std::time::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;
use std::thread::ThreadId;

thread_local! {
    static STATE: RefCell<LocalState> = RefCell::new(LocalState::default());
}

#[derive(Default, Clone)]
struct LocalState {
    is_mocked: bool,
    frozen: bool,
    time: Duration,
    shared_state: Arc<SharedState>,
}

#[derive(Default)]
struct SharedState {
    time: Mutex<Duration>,
    freeze_cond: Condvar,
    registry: TimedWaitRegistry,
}

#[derive(Default)]
struct TimedWaitRegistry {
    data: RwLock<HashMap<ThreadId, RegistryState>>,
}

#[derive(Default)]
struct RegistryState {
    block_mutex: Mutex<BlockState>,
    block_cond: Condvar,
}

#[derive(Default)]
struct BlockState {
    is_blocked: bool,
    block_id: usize,
}

impl TimedWaitRegistry {
    fn register_thread(&self) {
        self.data
            .write()
            .unwrap()
            .insert(thread::current().id(), RegistryState::default());
    }

    fn start_blocking_wait(&self) {
        let lock = self.data.read().unwrap();
        let registry_state = lock
            .get(&thread::current().id())
            .expect("chronobreak internal error: thread was not registered");
        let mut block_state = registry_state.block_mutex.lock().unwrap();
        block_state.block_id += 1;
        block_state.is_blocked = true;
        registry_state.block_cond.notify_all();
    }

    fn finish_blocking_wait(&self) {
        let lock = self.data.read().unwrap();
        let registry_state = lock
            .get(&thread::current().id())
            .expect("chronobreak internal error: thread was not registered");
        let mut block_state = registry_state.block_mutex.lock().unwrap();
        block_state.is_blocked = false;
    }

    fn wait_for_blocking(&self, id: ThreadId) {
        let lock = self.data.read().unwrap();
        let registry_state = lock
            .get(&id)
            .expect("chronobreak internal error: thread was not registered");
        let mut lock = registry_state.block_mutex.lock().unwrap();
        let block_id = lock.block_id + 1;
        while !lock.is_blocked && lock.block_id != block_id {
            lock = registry_state.block_cond.wait(lock).unwrap();
        }
    }
}

#[must_use]
pub struct ClockGuard {}

impl Drop for ClockGuard {
    fn drop(&mut self) {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.is_mocked = false;
            state.time = Duration::default();
            *state.shared_state.time.lock().unwrap() = Duration::default();
        });
    }
}

#[must_use]
pub struct UnfreezeGuard {
    was_frozen: bool,
}

impl Drop for UnfreezeGuard {
    fn drop(&mut self) {
        set_frozen(self.was_frozen)
    }
}

/// Asserts that the current local time is equal to an expression.
///
/// On panic, this macro will print the expected and actual local time.
#[macro_export]
macro_rules! assert_clock_eq {
    ($dur:expr) => ({
        match (&($dur),) {
            (dur,) => {
                if !(*dur == ::chronobreak::clock::get()) {
                    panic!(r#"clock assertion failed: `(expected == actual)`
 expected: `{:?}`,
   actual: `{:?}`"#, &*dur, ::chronobreak::clock::get())
                }
            }
        }
    });
    ($dur:expr,) => ({
        $crate::assert_clock_eq!($dur)
    });
    ($dur:expr, $($arg:tt)+) => ({
        match (&($dur),) {
            (dur,) => {
                if !(*dur == ::chronobreak::clock::get()) {
                    panic!(r#"clock assertion failed: `(expected == actual)`
 expected: `{:?}`,
   actual: `{:?}`: {}"#, &*dur, ::chronobreak::clock::get(),
                           $crate::format_args!($($arg)+))
                }
            }
        }
    });
}

/// Returns whether the clock is currently mocked on the current thread.
pub fn is_mocked() -> bool {
    STATE.with(|state| state.borrow().is_mocked)
}

/// Mocks the clock on the current thread. This function must **not** be called
/// again before the returned guard is dropped. Dropping the guard resets the
/// clock to the system clock and the internal values of the mocked clock to
/// Duration::default().
pub fn mocked() -> Result<ClockGuard, ()> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.is_mocked {
            Err(())
        } else {
            state.shared_state.registry.register_thread();
            state.is_mocked = true;
            Ok(ClockGuard {})
        }
    })
}

/// Freezes the clock on the current thread.
/// This causes all mocked routines on the current thread that perform
/// timed waiting to not increase the local clock automatically. Instead they
/// wait for the global clock to be manually advanced from another thread.
pub fn freeze() {
    set_frozen(true)
}

/// Returns wether the clock is frozen on the current thread.
pub fn is_frozen() -> bool {
    STATE.with(|state| state.borrow().frozen)
}

/// Unfreezes the clock on the current thread.
pub fn unfreeze() {
    set_frozen(false)
}

/// Unfreezes the clock on the current thread.
pub(crate) fn unfreeze_scoped() -> UnfreezeGuard {
    UnfreezeGuard {
        was_frozen: is_frozen(),
    }
}

fn set_frozen(frozen: bool) {
    STATE.with(|state| state.borrow_mut().frozen = frozen)
}

/// Selts the local and global clock to the given timestamp if it is greater
/// than the current local or global time, respectively.
pub fn advance_to(time: Instant) {
    if is_mocked() {
        if let Instant::Mocked(time) = time {
            STATE.with(|state| {
                let mut state = state.borrow_mut();
                if state.time < time {
                    state.time = time;
                }
                let shared_state = &state.shared_state;
                let mut global_time = shared_state.time.lock().unwrap();
                if *global_time < time {
                    if state.frozen {
                        shared_state.registry.start_blocking_wait();
                        while *global_time < time {
                            global_time = shared_state.freeze_cond.wait(global_time).unwrap();
                        }
                        shared_state.registry.finish_blocking_wait();
                    } else {
                        *global_time = time;
                        shared_state.freeze_cond.notify_all();
                    }
                }
            });
        } else {
            panic! {"chronobreak::clock::advance_to: the clock is mocked but the given Instant is not"}
        }
    } else {
        panic! {"chronobreak::clock::advance_to requires the clock to be mocked"};
    }
}

/// Temporarily unfreezes the clock, if frozen, then advances the clock to the
/// given timestamp. If the clock is not frozen, this function is equal to
/// advance_to.
pub(crate) fn unfreeze_advance_to(time: Instant) {
    let _guard = unfreeze_scoped();
    advance_to(time);
}

/// Advances the local clock by the given duration. Sets the global clock if
/// the new local time is greater.
pub fn advance(dur: Duration) {
    if is_mocked() {
        advance_to(Instant::now() + dur);
    } else {
        panic! {"chronobreak::clock::advance requires the clock to be mocked"};
    }
}

/// Returns the current local time.
pub fn get() -> Duration {
    if is_mocked() {
        STATE.with(|state| state.borrow().time)
    } else {
        panic! {"chronobreak::clock::get requires the clock to be mocked"};
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct ClockHandle(LocalState);

#[allow(dead_code)]
pub(crate) fn handle() -> ClockHandle {
    let mut handle = ClockHandle(STATE.with(|state| state.borrow().clone()));
    handle.0.frozen = false;
    handle
}

#[allow(dead_code)]
pub(crate) fn register_thread(handle: ClockHandle) {
    handle.0.shared_state.registry.register_thread();
    STATE.with(|state| *state.borrow_mut() = handle.0);
}

#[allow(dead_code)]
pub(crate) fn expect_blocking_advance_on(id: ThreadId) {
    STATE.with(|state| state.borrow().shared_state.registry.wait_for_blocking(id));
}

#[cfg(test)]
mod tests {
    use crate::clock;
    use crate::mock::std::{thread, time::Duration};

    #[test]
    fn main_thread_is_registered() {
        let _clock = clock::mocked().unwrap();
        clock::freeze();
        let main_thread = thread::current();
        thread::spawn(move || {
            clock::expect_blocking_advance_on(main_thread.id());
            clock::advance(Duration::from_millis(1))
        });
        clock::advance(Duration::from_millis(1));
    }

    #[test]
    fn frozen_wait_is_blocking() {
        let _clock = clock::mocked().unwrap();
        let thread = thread::spawn(move || {
            clock::freeze();
            clock::advance(Duration::from_nanos(1));
        });
        clock::expect_blocking_advance_on(thread.thread().id());
        clock::advance(Duration::from_nanos(1));
        thread.join().unwrap();
    }
}
