use std::collections::{BinaryHeap, HashMap};
use std::ops;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock, Weak};
use std::task::Waker;
use std::thread::{self, ThreadId};
use std::time::Duration;

/// Internal representation of the clock's current time.
#[derive(Debug, Default, Copy, Clone, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct Timepoint(Duration);

impl Timepoint {
    pub const START: Self = Self(Duration::from_secs(0));

    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        self.0.checked_add(duration).map(&Self)
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        self.0.checked_sub(duration).map(&Self)
    }

    pub fn duration_since(&self, earlier: Self) -> Duration {
        self.0
            .checked_sub(earlier.0)
            .expect("supplied timepoint is later than self")
    }

    pub fn checked_duration_since(self, earlier: Self) -> Option<Duration> {
        self.0.checked_sub(earlier.0)
    }

    pub fn saturating_duration_since(&self, earlier: Self) -> Duration {
        self.0.checked_sub(earlier.0).unwrap_or_default()
    }
}

impl ops::Add<Duration> for Timepoint {
    type Output = Timepoint;

    fn add(self, duration: Duration) -> Timepoint {
        Timepoint(self.0 + duration)
    }
}

impl ops::AddAssign<Duration> for Timepoint {
    fn add_assign(&mut self, rhs: Duration) {
        self.0.add_assign(rhs);
    }
}

impl ops::Sub<Duration> for Timepoint {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self {
        Timepoint(self.0.sub(rhs))
    }
}

impl ops::SubAssign<Duration> for Timepoint {
    fn sub_assign(&mut self, rhs: Duration) {
        self.0.sub_assign(rhs);
    }
}

/// State of the shared clock.
#[derive(Default)]
pub struct SharedClock {
    /// The current shared time.
    time: Mutex<Timepoint>,
    /// Condvar, which all threads who attempt to advance the shared clock
    /// while frozen will wait on.
    freeze_cond: Condvar,
    /// Per-thread data for notifying and waiting on timed waits.
    timed_waits: Arc<TimedWaitData>,
    /// Queue of wakers that have to be executed as soon as the clock reaches
    /// some given time.
    wakers: Mutex<BinaryHeap<TimedWaker>>,
}

impl SharedClock {
    pub fn register_thread(&self) {
        self.timed_waits
            .write()
            .unwrap()
            .insert(thread::current().id(), Default::default());
    }

    pub fn advance_to(&self, time: Timepoint) {
        let mut global_time = self.time.lock().unwrap();
        if *global_time < time {
            let _guard = self.notify_timed_wait();
            while *global_time < time {
                global_time = self.freeze_cond.wait(global_time).unwrap();
            }
        }
    }

    pub fn unfreeze_advance_to(&self, time: Timepoint) {
        let mut global_time = self.time.lock().unwrap();
        if *global_time < time {
            *global_time = time;
            self.freeze_cond.notify_all();
            let mut wakers = self.wakers.lock().unwrap();
            while let Some(timed_waker) = wakers.peek() {
                if timed_waker.timeout <= time {
                    if let Some(waker) = wakers.pop().unwrap().waker.upgrade() {
                        waker.wake_by_ref();
                    }
                } else {
                    break;
                }
            }
        }
    }

    pub fn notify_timed_wait(&self) -> TimedWaitGuard {
        let lock = self.timed_waits.read().unwrap();
        let thread_info = lock
            .get(&thread::current().id())
            .expect("chronobreak internal error: thread was not registered");
        let _lock = thread_info.1.lock().unwrap();
        thread_info.0.fetch_add(1, Ordering::SeqCst);
        thread_info.2.notify_all();
        TimedWaitGuard::new(self.timed_waits.clone())
    }

    pub fn expect_timed_wait_on(&self, id: ThreadId) {
        let lock = self.timed_waits.read().unwrap();
        let timed_waits = lock
            .get(&id)
            .expect("chronobreak internal error: thread was not registered");
        let mut lock = timed_waits.1.lock().unwrap();
        while timed_waits.0.load(Ordering::SeqCst) == 0 {
            lock = timed_waits.2.wait(lock).unwrap();
        }
    }

    pub fn register_timed_waker(
        &self,
        waker: Waker,
        timeout: Timepoint,
    ) -> (Option<TimedWakerHandle>, Timepoint) {
        let current_time = *self.time.lock().unwrap();
        if current_time < timeout {
            let mut wakers = self.wakers.lock().unwrap();
            let guard = self.notify_timed_wait();
            let result = TimedWakerHandle {
                waker: Arc::new(waker),
                guard,
            };
            wakers.push(TimedWaker {
                waker: Arc::downgrade(&result.waker),
                timeout,
            });
            (Some(result), current_time)
        } else {
            (None, current_time)
        }
    }
}

type TimedWaitData = RwLock<HashMap<ThreadId, (AtomicUsize, Mutex<()>, Condvar)>>;

/// A RAII implementation for a timed wait. When this guard is dropped, the
/// timed wait counter for the thread it was created on will be decreased.
#[must_use = "if unused the timed wait state will be immediately reset"]
pub struct TimedWaitGuard {
    created_on: ThreadId,
    data: Arc<TimedWaitData>,
}

impl TimedWaitGuard {
    pub fn new(data: Arc<TimedWaitData>) -> Self {
        Self {
            created_on: thread::current().id(),
            data,
        }
    }
}

impl Drop for TimedWaitGuard {
    fn drop(&mut self) {
        let lock = self.data.read().unwrap();
        let thread_info = lock
            .get(&self.created_on)
            .expect("chronobreak internal error: thread was not registered");
        thread_info.0.fetch_sub(1, Ordering::SeqCst);
    }
}

/// A waker with a associated execution time.
struct TimedWaker {
    waker: Weak<Waker>,
    timeout: Timepoint,
}

impl Ord for TimedWaker {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.timeout.cmp(&rhs.timeout)
    }
}

impl PartialOrd<TimedWaker> for TimedWaker {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        self.timeout.partial_cmp(&rhs.timeout)
    }
}

impl Eq for TimedWaker {}

impl PartialEq<TimedWaker> for TimedWaker {
    fn eq(&self, rhs: &Self) -> bool {
        self.timeout.eq(&rhs.timeout)
    }
}

/// Handle to a timed waker. If this handle is dropped, the waker will no longer
/// be executed.
pub struct TimedWakerHandle {
    waker: Arc<Waker>,
    #[allow(dead_code)]
    guard: TimedWaitGuard,
}

impl TimedWakerHandle {
    pub fn waker(&self) -> Waker {
        (*self.waker).clone()
    }
}
