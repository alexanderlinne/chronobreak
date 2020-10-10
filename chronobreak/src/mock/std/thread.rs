use crate::clock::{auto_inc, manual, ClockStrategy};
use crate::mock::std::sync::{Arc, Mutex};
use crate::mock::std::time::*;
use std::thread;

pub use std::thread::{panicking, Builder};

pub fn sleep(dur: Duration) {
    match ClockStrategy::current() {
        ClockStrategy::Sys => thread::sleep(dur),
        ClockStrategy::Manual => {}
        ClockStrategy::AutoInc => {
            auto_inc::fetch_add(dur);
        }
    }
}

pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let strategy = ClockStrategy::raw();
    let manual = manual::raw();
    let auto_inc = auto_inc::raw();
    let join_cell = Arc::new(Mutex::new(None));
    let join_cell_weak = Arc::downgrade(&join_cell);
    let handle = thread::spawn(move || {
        ClockStrategy::from_raw(strategy);
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

#[cfg(test)]
mod tests {
    use crate::clock::*;
    use crate::mock::std::sync::{Arc, Barrier};
    use crate::mock::std::thread;
    use crate::mock::std::time::*;

    #[test]
    fn manual_transfers_on_thread_spawn() {
        let _clock = ClockStrategy::set(ClockStrategy::Manual).unwrap();
        manual::fetch_add(Duration::from_nanos(1));
        thread::spawn(move || {
            assert_eq!(manual::get(), Duration::from_nanos(1));
        })
        .join()
        .unwrap();
    }

    #[test]
    fn manual_clock_is_global() {
        let _clock = ClockStrategy::set(ClockStrategy::Manual).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let barrier2 = barrier.clone();
        let thread = thread::spawn(move || {
            barrier2.wait();
            assert_eq!(manual::get(), Duration::from_nanos(1));
        });
        manual::fetch_add(Duration::from_nanos(1));
        barrier.wait();
        thread.join().unwrap();
    }

    #[test]
    fn auto_inc_transfers_on_thread_spawn() {
        let _clock = ClockStrategy::set(ClockStrategy::AutoInc).unwrap();
        auto_inc::fetch_add(Duration::from_nanos(1));
        thread::spawn(move || {
            assert_eq!(auto_inc::get(), Duration::from_nanos(1));
        })
        .join()
        .unwrap();
    }

    #[test]
    fn auto_inc_thread_sleep() {
        let _clock = ClockStrategy::set(ClockStrategy::AutoInc).unwrap();
        thread::sleep(Duration::from_nanos(1));
        assert_eq!(auto_inc::get(), Duration::from_nanos(1));
    }

    #[test]
    fn auto_inc_is_not_global() {
        let _clock = ClockStrategy::set(ClockStrategy::AutoInc).unwrap();
        // Don't use mock thread::spawn here!
        std::thread::spawn(move || {
            thread::sleep(Duration::from_nanos(1));
        })
        .join()
        .unwrap();
        assert_eq!(auto_inc::get(), Duration::from_nanos(0));
    }

    #[test]
    fn auto_inc_thread_join_sync() {
        let _clock = ClockStrategy::set(ClockStrategy::AutoInc).unwrap();
        thread::spawn(move || {
            thread::sleep(Duration::from_nanos(1));
        })
        .join()
        .unwrap();
        assert_eq!(auto_inc::get(), Duration::from_nanos(1));
    }
}
