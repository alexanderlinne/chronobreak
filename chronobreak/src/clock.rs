use crate::mock::std::time::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
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
    block_id: AtomicUsize,
    block_mutex: Mutex<()>,
    block_cond: Condvar,
}

impl TimedWaitRegistry {
    fn register_thread(&self) {
        self.data
            .write()
            .unwrap()
            .insert(thread::current().id(), RegistryState::default());
    }

    fn notify_blocking_wait(&self) {
        let lock = self.data.read().unwrap();
        let registry_state = lock
            .get(&thread::current().id())
            .expect("chronobreak internal error: thread was not registered");
        registry_state.block_id.fetch_add(1, Ordering::SeqCst);
        registry_state.block_cond.notify_all();
    }

    fn wait_for_blocking(&self, id: ThreadId) {
        let lock = self.data.read().unwrap();
        let registry_state = lock
            .get(&id)
            .expect("chronobreak internal error: thread was not registered");
        let mut lock = registry_state.block_mutex.lock().unwrap();
        let block_id = registry_state.block_id.load(Ordering::SeqCst);
        while !registry_state.block_id.load(Ordering::SeqCst) == block_id + 1 {
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

#[macro_export]
macro_rules! assert_clock_eq {
    ($dur:expr) => {
        if ::chronobreak::clock::is_mocked() {
            assert_eq!(::chronobreak::clock::get(), $dur);
        } else {
            panic! {"assert_clock_eq! {...} needs the clock to be mocked!"};
        }
    };
}

// Returns whether the clock is currently mocked on the current thread.
pub fn is_mocked() -> bool {
    STATE.with(|state| state.borrow().is_mocked)
}

// Mocks the clock on the current thread. This function must **not** be called
// again before the returned guard is dropped. Dropping the guard resets the
// clock to the system clock and the internal values of the mocked clock to
// Duration::default().
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

// Freezes the clock on the current thread.
// This causes all mocked routines on the current thread that perform
// timed waiting to not increase the local clock automatically. Instead they
// wait for the global clock to be manually advanced from another thread.
pub fn freeze() {
    set_frozen(true)
}

// Returns wether the clock is frozen on the current thread.
pub fn is_frozen() -> bool {
    STATE.with(|state| state.borrow().frozen)
}

// Unfreezes the clock on the current thread.
pub fn unfreeze() {
    set_frozen(false)
}

// Unfreezes the clock on the current thread.
pub(crate) fn unfreeze_scoped() -> UnfreezeGuard {
    UnfreezeGuard {
        was_frozen: is_frozen(),
    }
}

fn set_frozen(frozen: bool) {
    STATE.with(|state| state.borrow_mut().frozen = frozen)
}

// Selts the local and global clock to the given timestamp if it is greater
// than the current local or global time, respectively.
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
                        shared_state.registry.notify_blocking_wait();
                        while *global_time < time {
                            global_time = shared_state.freeze_cond.wait(global_time).unwrap();
                        }
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

// Advances the local clock by the given duration. Sets the global clock if
// the new local time is greater.
pub fn advance(dur: Duration) {
    if is_mocked() {
        advance_to(Instant::now() + dur);
    } else {
        panic! {"chronobreak::clock::advance requires the clock to be mocked"};
    }
}

// Returns the current local time.
pub fn get() -> Duration {
    if is_mocked() {
        STATE.with(|state| state.borrow().time)
    } else {
        panic! {"chronobreak::clock::get requires the clock to be mocked"};
    }
}

// Synchronizes the global and local time. Sets both to the greater of both
// timestamps.
pub fn synchroize() {
    if is_mocked() {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let mut global_time = state.shared_state.time.lock().unwrap();
            let current_time = std::cmp::max(*global_time, state.time);
            *global_time = current_time;
            drop(global_time);
            state.time = current_time;
        });
    } else {
        panic! {"chronobreak::clock::synchroize requires the clock to be mocked"};
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct ClockHandle(LocalState);

#[allow(dead_code)]
pub(crate) fn handle() -> ClockHandle {
    ClockHandle(STATE.with(|state| state.borrow().clone()))
}

#[allow(dead_code)]
pub(crate) fn register_thread(handle: ClockHandle) {
    handle.0.shared_state.registry.register_thread();
    STATE.with(|state| *state.borrow_mut() = handle.0);
}

pub(crate) fn expect_blocking_wait_on(id: ThreadId) {
    STATE.with(|state| state.borrow().shared_state.registry.wait_for_blocking(id));
}
