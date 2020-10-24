use crate::mock::std::time::*;
use crate::shared_clock::{SharedClock, TimedWakerHandle};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread::ThreadId;

thread_local! {
    static STATE: RefCell<Option<LocalState>> = RefCell::new(None);
}

#[derive(Default, Clone)]
struct LocalState {
    frozen: bool,
    time: Duration,
    shared_clock: Arc<SharedClock>,
}

/// A RAII guard returned by [`mock`](fn.mock.html). When this structure is
/// dropped, the mocked clock is destroyed.
#[must_use = "if unused the mocked clock will be immediately dropped"]
pub struct ClockGuard {}

impl Drop for ClockGuard {
    fn drop(&mut self) {
        STATE.with(|state| {
            *state.borrow_mut() = None;
        });
    }
}

#[must_use = "if unused the mocked clock will immediately be unfrozen"]
pub(crate) struct UnfreezeGuard {
    was_frozen: bool,
}

impl Drop for UnfreezeGuard {
    fn drop(&mut self) {
        set_frozen(self.was_frozen)
    }
}

pub(crate) struct DelayFuture {
    timeout: Instant,
    waker_handle: Option<TimedWakerHandle>,
}

impl DelayFuture {
    pub fn new(delay: Duration) -> Self {
        Self {
            timeout: Instant::now() + delay,
            waker_handle: None,
        }
    }

    pub fn reset(&mut self, delay: Duration) {
        STATE.with(|state| {
            let state = state.borrow();
            let shared_clock = &state
                .as_ref()
                .expect("chronobreak::DelayFuture::poll requires the clock to be mocked")
                .shared_clock;
            self.timeout = Instant::now() + delay;
            if let Some(handle) = self.waker_handle.take() {
                self.waker_handle = shared_clock
                    .register_timed_waker(handle.waker(), self.timeout)
                    .0;
            }
        })
    }
}

impl Future for DelayFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if !is_frozen() {
            advance_to(self.timeout);
            return Poll::Ready(());
        }
        let (handle, current_time) = STATE.with(|state| {
            let state = state.borrow();
            let shared_clock = &state
                .as_ref()
                .expect("chronobreak::DelayFuture::poll requires the clock to be mocked")
                .shared_clock;
            shared_clock.register_timed_waker(cx.waker().clone(), self.timeout)
        });
        let this = Pin::into_inner(self);
        this.waker_handle = handle;
        unfreeze_advance_to(current_time);
        if this.waker_handle.is_some() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

/// Returns whether the clock is currently mocked on the current thread.
pub fn is_mocked() -> bool {
    STATE.with(|state| state.borrow().is_some())
}

/// Mocks the clock on the current thread. This function must **not** be called
/// again before the returned guard is dropped. Dropping the guard resets the
/// clock to the system clock and the internal values of the mocked clock to
/// Duration::default().
pub fn mock() -> Result<ClockGuard, ()> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.is_some() {
            Err(())
        } else {
            let init = LocalState::default();
            init.shared_clock.register_thread();
            *state = Some(init);
            Ok(ClockGuard {})
        }
    })
}

/// Similar to [`mock`](fn.mock.html) but also freezes the clock on the
/// current thread.
/// This causes all mocked routines on the current thread that perform
/// timed waiting to not increase the local clock automatically. Instead they
/// wait for the global clock to be manually advanced from another thread.
pub fn frozen() -> Result<ClockGuard, ()> {
    let result = mock();
    set_frozen(true);
    result
}

/// Returns wether the clock is frozen on the current thread.
pub fn is_frozen() -> bool {
    STATE.with(|state| {
        state
            .borrow()
            .as_ref()
            .expect("chronobreak::clock::is_frozen requires the clock to be mocked")
            .frozen
    })
}

/// Unfreezes the clock on the current thread until the returned guard is dropped.
pub(crate) fn unfreeze_scoped() -> UnfreezeGuard {
    UnfreezeGuard {
        was_frozen: is_frozen(),
    }
}

fn set_frozen(frozen: bool) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .as_mut()
            .expect("chronobreak::clock::set_frozen requires the clock to be mocked")
            .frozen = frozen
    })
}

/// Sets the local and shared clock to the given timestamp if it is greater
/// than the current local or global time, respectively.
pub fn advance_to(time: Instant) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let mut state = state
            .as_mut()
            .expect("chronobreak::clock::advance_to requires the clock to be mocked");
        if let Instant::Mocked(time) = time {
            if state.time < time {
                state.time = time;
            }
        } else {
            panic! {"chronobreak::clock::advance_to requires a mocked Instant"}
        }
        if state.frozen {
            state.shared_clock.advance_to(time);
        } else {
            state.shared_clock.unfreeze_advance_to(time);
        }
    });
}

/// Temporarily unfreezes the clock, if frozen, then advances the clock to the
/// given timestamp. If the clock is not frozen, this function is equal to
/// advance_to.
pub(crate) fn unfreeze_advance_to(time: Instant) {
    let _guard = unfreeze_scoped();
    advance_to(time);
}

/// Advances the local clock by the given duration. Sets the shared clock if
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
    STATE.with(|state| {
        state
            .borrow()
            .as_ref()
            .expect("chronobreak::clock::get requires the clock to be mocked")
            .time
    })
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct ClockHandle(Option<LocalState>);

#[allow(dead_code)]
pub(crate) fn handle() -> ClockHandle {
    let mut handle = ClockHandle(STATE.with(|state| state.borrow().clone()));
    if let Some(local_state) = handle.0.as_mut() {
        local_state.frozen = false;
    }
    handle
}

#[allow(dead_code)]
pub(crate) fn register_thread(handle: ClockHandle) {
    if let Some(local_state) = handle.0.as_ref() {
        local_state.shared_clock.register_thread();
    }
    STATE.with(|state| *state.borrow_mut() = handle.0);
}

#[allow(dead_code)]
pub(crate) fn expect_timed_wait_on(id: ThreadId) {
    STATE.with(|state| {
        state
            .borrow()
            .as_ref()
            .expect(
                "chronobreak::clock::expect_blocking_advance_on requires the clock to be mocked",
            )
            .shared_clock
            .expect_timed_wait_on(id)
    });
}

#[cfg(test)]
mod tests {
    use crate::clock;
    use crate::mock::std::{thread, time::Duration};

    #[test]
    fn main_thread_is_registered() {
        let _clock = clock::frozen().unwrap();
        let main_thread = thread::current();
        thread::spawn(move || {
            clock::expect_timed_wait_on(main_thread.id());
            clock::advance(Duration::from_millis(1))
        });
        clock::advance(Duration::from_millis(1));
    }

    #[test]
    fn frozen_wait_is_blocking() {
        let _clock = clock::frozen().unwrap();
        let main_thread = thread::current();
        let thread = thread::spawn(move || {
            main_thread.expect_timed_wait();
            clock::advance(Duration::from_nanos(1));
        });
        clock::advance(Duration::from_nanos(1));
        thread.join().unwrap();
    }
}
