use crate::error::ChronobreakError;
use crate::shared_clock::{SharedClock, TimedWakerHandle};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread::ThreadId;
use std::time::Duration;

pub use crate::shared_clock::Timepoint;

thread_local! {
    /// State of the mocked clock. None if the clock is not mocked.
    static STATE: RefCell<Option<LocalClock>> = RefCell::new(None);
}

/// State of the local clock.
#[derive(Default, Clone)]
struct LocalClock {
    /// true if the clock is frozen on this thread, otherwise false.
    frozen: bool,
    /// The current local time.
    time: Timepoint,
    /// The shared clock.
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

/// A RAII guard returned by [`unfreeze_scoped`](fn.unfreeze_scoped.html).
/// When this structure is dropped, the clock will be frozen if it was frozen
/// during creation.
#[must_use = "if unused the mocked clock will immediately be unfrozen"]
pub(crate) struct UnfreezeGuard {
    was_frozen: bool,
}

impl Drop for UnfreezeGuard {
    fn drop(&mut self) {
        set_frozen(self.was_frozen)
    }
}

/// A mocked version of a future that becomes ready after a given delay.
///
/// # Non-frozen behaviour
///
/// This future will be ready immediately and advance the clock by the given
/// delay. (Time of creation of the instance plus the givendelay.)
///
/// # Frozen behaviour
///
/// When polled, given waker will be registered to be called as soon as any other
/// thread advances the shared clock past the time of creation of this instance
/// plus the given delay. If the shared clock has already passed this time,
/// the future will be ready immediately.
pub struct DelayFuture {
    timeout: Timepoint,
    waker_handle: Option<TimedWakerHandle>,
}

impl DelayFuture {
    pub fn new(delay: Duration) -> Self {
        Self {
            timeout: get() + delay,
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
            self.timeout = get() + delay;
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
        let _guard = unfreeze_scoped();
        advance_to(current_time);
        if this.waker_handle.is_some() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

/// Returns whether the clock is currently mocked on the current thread.
pub(crate) fn is_mocked() -> bool {
    STATE.with(|state| state.borrow().is_some())
}

/// Mocks the clock on the current thread. This function must **not** be called
/// again before the returned guard is dropped. Dropping the guard resets the
/// clock to the system clock and the internal values of the mocked clock to
/// Duration::default().
pub fn mock() -> Result<ClockGuard, ChronobreakError> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.is_some() {
            Err(ChronobreakError::AlreadyInitialized)
        } else {
            let init = LocalClock::default();
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
pub fn frozen() -> Result<ClockGuard, ChronobreakError> {
    let result = mock();
    set_frozen(true);
    result
}

/// Returns wether the clock is frozen on the current thread.
///
/// # Panics
///
/// This function panics if the clock is not mocked on the current thread.
fn is_frozen() -> bool {
    STATE.with(|state| {
        state
            .borrow()
            .as_ref()
            .expect("chronobreak::clock::is_frozen requires the clock to be mocked")
            .frozen
    })
}

/// Unfreezes the clock on the current thread until the returned guard is dropped.
///
/// # Panics
///
/// This function panics if the clock is not mocked on the current thread.
pub(crate) fn unfreeze_scoped() -> UnfreezeGuard {
    UnfreezeGuard {
        was_frozen: is_frozen(),
    }
}

/// Sets the frozen flag for the current thread.
///
/// # Panics
///
/// This function panics if the clock is not mocked on the current thread.
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
///
/// # Panics
///
/// This function panics if the clock is not mocked on the current thread and
/// if the given time is not a mocked Instant. (Any Instant created without the
/// clock being mocked.)
fn advance_to(time: Timepoint) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let mut state = state
            .as_mut()
            .expect("chronobreak::clock::advance_to requires the clock to be mocked");
        if state.time < time {
            state.time = time;
        }
        if state.frozen {
            state.shared_clock.advance_to(time);
        } else {
            state.shared_clock.unfreeze_advance_to(time);
        }
    });
}

/// Advances the local clock by the given duration. Sets the shared clock if
/// the new local time is greater.
///
/// # Panics
///
/// This function panics if the clock is not mocked on the current thread.
pub fn advance(dur: Duration) {
    if is_mocked() {
        advance_to(get() + dur);
    } else {
        panic! {"chronobreak::clock::advance requires the clock to be mocked"};
    }
}

/// Returns the current local time.
///
/// # Panics
///
/// This function panics if the clock is not mocked on the current thread.
pub fn get() -> Timepoint {
    STATE.with(|state| {
        state
            .borrow()
            .as_ref()
            .expect("chronobreak::clock::get requires the clock to be mocked")
            .time
    })
}

/// A handle that can be used to register a new thread to the same shared clock
/// that the thread which created this handle is registered to.
#[derive(Clone)]
pub(crate) struct RegistrationHandle(Option<LocalClock>);

/// Returns a handle to the mocked clock of the current thread. An empty handle
/// is returned if the clock is not mocked.
pub(crate) fn registration_handle() -> RegistrationHandle {
    let mut handle = RegistrationHandle(STATE.with(|state| state.borrow().clone()));
    if let Some(local_state) = handle.0.as_mut() {
        local_state.frozen = false;
    }
    handle
}

/// Registers the given handle for the current thread. The local clock will
/// have the same time as the local clock of the thread on which
/// [`registration_handle`](fn.registration_handle.html) has been called but will not be frozen,
/// independently of whether the original thread had a frozen clock or not.
/// After the call, both threads will share a common shared clock.
pub(crate) fn register_thread(handle: RegistrationHandle) {
    if let Some(local_state) = handle.0.as_ref() {
        local_state.shared_clock.register_thread();
    }
    STATE.with(|state| *state.borrow_mut() = handle.0);
}

/// A handle that can be used to synchronize a thread's local clock to the time
/// at which this handle was created.
#[derive(Clone, Copy)]
pub(crate) struct SyncHandle(Option<Timepoint>);

/// Returns a synchronization handle with the calling thread's current local
/// time. An empty handle is returned if the clock is not mocked.
pub(crate) fn sync_handle() -> SyncHandle {
    if is_mocked() {
        SyncHandle(Some(get()))
    } else {
        SyncHandle(None)
    }
}

/// Synchronizes the calling thread's local clock with the synchronization
/// handle.
pub(crate) fn sync_with(handle: SyncHandle) {
    if let Some(timepoint) = handle.0 {
        advance_to(timepoint);
    } else if is_mocked() {
        panic! {"chronobreak::clock::sync_with called with an unmocked SyncHandle on a mocked clock"}
    }
}

/// Blocks the current thread until the thread with the given thread id enters
/// a timed wait.
///
/// # Panics
///
/// This function panics if the clock is not mocked on the current thread.
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
    use crate::mock::std::{thread, time::*};

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
