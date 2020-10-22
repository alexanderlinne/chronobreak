use crate::mock::std::time::*;
use std::collections::{BinaryHeap, HashMap};
use std::sync::{Arc, Condvar, Mutex, RwLock, Weak};
use std::task::Waker;
use std::thread;
use std::thread::ThreadId;

#[derive(Default)]
pub struct SharedClock {
    time: Mutex<Duration>,
    freeze_cond: Condvar,
    blocking: BlockingWaitData,
    waker: Mutex<BinaryHeap<TimedWaker>>,
}

impl SharedClock {
    pub fn register_thread(&self) {
        self.blocking
            .write()
            .unwrap()
            .insert(thread::current().id(), Default::default());
    }

    pub fn advance_to(&self, time: Instant) {
        if let Instant::Mocked(time) = time {
            let mut global_time = self.time.lock().unwrap();
            if *global_time < time {
                let _guard = self.blocking_wait();
                while *global_time < time {
                    global_time = self.freeze_cond.wait(global_time).unwrap();
                }
            }
        } else {
            panic! {"chronobreak::shared_clock::advance_to requires a mocked Instant"}
        }
    }

    pub fn unfreeze_advance_to(&self, time: Instant) {
        if let Instant::Mocked(time) = time {
            let mut global_time = self.time.lock().unwrap();
            if *global_time < time {
                *global_time = time;
                self.freeze_cond.notify_all();
                let mut wakers = self.waker.lock().unwrap();
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
        } else {
            panic! {"chronobreak::shared_clock::unfreeze_advance_to requires a mocked Instant"}
        }
    }

    pub fn blocking_wait(&self) -> BlockingWaitGuard<'_> {
        let lock = self.blocking.read().unwrap();
        let blocking_data = lock
            .get(&thread::current().id())
            .expect("chronobreak internal error: thread was not registered");
        let mut block_state = blocking_data.0.lock().unwrap();
        block_state.1 += 1;
        block_state.0 = true;
        blocking_data.1.notify_all();
        BlockingWaitGuard {
            data: &self.blocking,
        }
    }

    pub fn wait_for_blocking(&self, id: ThreadId) {
        let lock = self.blocking.read().unwrap();
        let blocking_data = lock
            .get(&id)
            .expect("chronobreak internal error: thread was not registered");
        let mut lock = blocking_data.0.lock().unwrap();
        let block_id = lock.1 + 1;
        while !lock.0 && lock.1 != block_id {
            lock = blocking_data.1.wait(lock).unwrap();
        }
    }

    /// Registers the given waker to be woken as soon as the global clock passes the given timeout. If the
    /// timeout is in the future compared to the global time, a handle to the waker and the current
    /// global time is returned. Otherwise, i.e. when the global time has already passed the timeout,
    /// None is returned.
    ///
    /// If the returned handle is dropped, the waker will no longer be woken.
    pub fn register_timed_waker(
        &self,
        waker: Waker,
        timeout: Instant,
    ) -> (Option<TimedWakerHandle>, Instant) {
        if let Instant::Mocked(timeout) = timeout {
            let current_time = *self.time.lock().unwrap();
            if current_time <= timeout {
                let result = TimedWakerHandle(Arc::new(waker));
                self.waker.lock().unwrap().push(TimedWaker {
                    waker: Arc::downgrade(&result.0),
                    timeout,
                });
                (Some(result), Instant::Mocked(current_time))
            } else {
                (None, Instant::Mocked(current_time))
            }
        } else {
            panic! {"chronobreak::shared_clock::register_timed_waker requires a mocked Instant"}
        }
    }
}

type BlockingWaitData = RwLock<HashMap<ThreadId, (Mutex<(bool, usize)>, Condvar)>>;

pub struct BlockingWaitGuard<'a> {
    data: &'a BlockingWaitData,
}

impl<'a> Drop for BlockingWaitGuard<'a> {
    fn drop(&mut self) {
        let lock = self.data.read().unwrap();
        let blocking_data = lock
            .get(&thread::current().id())
            .expect("chronobreak internal error: thread was not registered");
        let mut block_state = blocking_data.0.lock().unwrap();
        block_state.0 = false;
    }
}

struct TimedWaker {
    waker: Weak<Waker>,
    timeout: Duration,
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

pub struct TimedWakerHandle(Arc<Waker>);

impl TimedWakerHandle {
    pub fn waker(&self) -> Waker {
        (*self.0).clone()
    }
}
