use crate::clock;
use crate::clock::{auto_inc, manual};
use crate::mock::std::sync::{Arc, Mutex};
use crate::mock::std::time::*;
use std::thread;

pub use std::thread::{panicking, Builder};

pub fn sleep(dur: Duration) {
    match_clock_strategy! {
        Sys => thread::sleep(dur),
        Manual => {},
        AutoInc => {
            clock::fetch_add(dur);
        },
    }
}

pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let strategy = clock::raw();
    let manual = manual::raw();
    let auto_inc = auto_inc::raw();
    let join_cell = Arc::new(Mutex::new(None));
    let join_cell_weak = Arc::downgrade(&join_cell);
    let handle = thread::spawn(move || {
        clock::from_raw(strategy);
        manual::from_raw(manual);
        auto_inc::from_raw(auto_inc);
        let result = f();
        if let Some(cell) = join_cell_weak.upgrade() {
            *cell.lock().unwrap() = Some(auto_inc::raw());
        }
        result
    });
    JoinHandle(join_cell, handle)
}

pub struct JoinHandle<T>(Arc<Mutex<Option<auto_inc::Raw>>>, thread::JoinHandle<T>);

impl<T> JoinHandle<T> {
    pub fn join(self) -> thread::Result<T> {
        let result = self.1.join();
        if let Some(auto_inc) = *self.0.lock().unwrap() {
            auto_inc::from_raw(auto_inc);
        }
        result
    }
}
