use crate::clock;
use crate::mock::std::sync::{Arc, Barrier, Mutex};
use crate::mock::std::time::*;
use std::thread;

pub use std::thread::{panicking, Builder, ThreadId};

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

    pub fn expect_blocking_wait(&self) {
        clock::expect_blocking_wait_on(self.0.id())
    }
}

pub fn current() -> Thread {
    Thread(thread::current())
}

pub fn sleep(dur: Duration) {
    if clock::is_mocked() {
        clock::advance(dur);
    } else {
        thread::sleep(dur);
    }
}

pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let clock_handle = clock::handle();
    let join_cell = Arc::new(Mutex::new(None));
    let join_cell_weak = Arc::downgrade(&join_cell);
    let barrier = Arc::new(Barrier::new(2));
    let barrier2 = barrier.clone();
    let handle = thread::spawn(move || {
        clock::register_thread(clock_handle);
        barrier2.wait();
        let result = f();
        if let Some(cell) = join_cell_weak.upgrade() {
            *cell.lock().unwrap() = Some(Instant::now());
        }
        result
    });
    barrier.wait();
    JoinHandle(join_cell, handle)
}

pub struct JoinHandle<T>(Arc<Mutex<Option<Instant>>>, thread::JoinHandle<T>);

impl<T> JoinHandle<T> {
    pub fn join(self) -> thread::Result<T> {
        let result = self.1.join();
        if clock::is_mocked() {
            if let Some(time) = *self.0.lock().unwrap() {
                let _guard = clock::unfreeze_scoped();
                clock::advance_to(time);
            }
        }
        result
    }

    pub fn expect_blocking_wait(&self) {
        clock::expect_blocking_wait_on(self.1.thread().id())
    }
}
