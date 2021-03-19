use crate::clock;
use crate::mock::std::sync::{Arc, Barrier, Mutex};
use crate::mock::std::time::Duration;
use std::thread;

#[allow(deprecated)]
pub use std::thread::{
    panicking, park, park_timeout, park_timeout_ms, sleep_ms, yield_now, AccessError, LocalKey,
    Result, ThreadId,
};

/// **Mock** of [`std::thread::Thread`](https://doc.rust-lang.org/std/thread/struct.Thread.html)
#[derive(Clone, Debug)]
pub struct Thread(thread::Thread);

impl Thread {
    pub fn unpark(&self) {
        self.0.unpark();
    }

    pub fn id(&self) -> ThreadId {
        self.0.id()
    }

    pub fn name(&self) -> Option<&str> {
        self.0.name()
    }

    #[cfg(feature = "extended-apis")]
    #[cfg_attr(docsrs, doc(cfg(feature = "extended-apis")))]
    pub fn expect_timed_wait(&self) {
        clock::expect_timed_wait_on(self.0.id())
    }
}

/// **Mock** of [`std::thread::current`](https://doc.rust-lang.org/std/thread/fn.current.html)
pub fn current() -> Thread {
    Thread(thread::current())
}

/// **Mock** of [`std::thread::sleep`](https://doc.rust-lang.org/std/thread/fn.sleep.html)
pub fn sleep(dur: Duration) {
    if clock::is_mocked() {
        clock::advance(dur);
    } else {
        thread::sleep(dur);
    }
}

/// **Mock** of [`std::thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html)
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let clock_handle = clock::registration_handle();
    let join_cell = Arc::new(Mutex::new(None));
    let join_cell_weak = Arc::downgrade(&join_cell);
    let barrier = Arc::new(Barrier::new(2));
    let barrier2 = barrier.clone();
    let handle = thread::spawn(move || {
        clock::register_thread(clock_handle);
        barrier2.wait();
        let result = f();
        if let Some(cell) = join_cell_weak.upgrade() {
            *cell.lock().unwrap() = Some(clock::sync_handle());
        }
        result
    });
    barrier.wait();
    let thread = handle.thread().clone();
    JoinHandle(join_cell, handle, Thread(thread))
}

/// **Mock** of [`std::thread::JoinHandle`](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html)
pub struct JoinHandle<T>(
    Arc<Mutex<Option<clock::SyncHandle>>>,
    thread::JoinHandle<T>,
    Thread,
);

impl<T> JoinHandle<T> {
    pub fn thread(&self) -> &Thread {
        &self.2
    }

    pub fn join(self) -> thread::Result<T> {
        let result = self.1.join();
        if clock::is_mocked() {
            if let Some(sync_handle) = *self.0.lock().unwrap() {
                let _guard = clock::unfreeze_scoped();
                clock::sync_with(sync_handle);
            }
        }
        result
    }

    #[cfg(feature = "extended-apis")]
    #[cfg_attr(docsrs, doc(cfg(feature = "extended-apis")))]
    pub fn expect_timed_wait(&self) {
        clock::expect_timed_wait_on(self.2.id())
    }
}
