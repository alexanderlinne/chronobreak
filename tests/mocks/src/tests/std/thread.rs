use chronobreak::clock;
use std::sync::Barrier;

#[chronobreak]
mod mock {
    pub use std::sync::Arc;
    pub use std::thread;
    pub use std::time::*;
}
use mock::*;

#[test]
fn manual_transfers_on_thread_spawn() {
    let _clock = clock::manual().unwrap();
    clock::fetch_add(Duration::from_nanos(1));
    thread::spawn(move || {
        assert_clock_eq!(Duration::from_nanos(1));
    })
    .join()
    .unwrap();
}

#[test]
fn manual_clock_is_global() {
    let _clock = clock::manual().unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let barrier2 = barrier.clone();
    let thread = thread::spawn(move || {
        barrier2.wait();
        assert_clock_eq!(Duration::from_nanos(1));
    });
    clock::fetch_add(Duration::from_nanos(1));
    barrier.wait();
    thread.join().unwrap();
}

#[test]
fn auto_inc_transfers_on_thread_spawn() {
    let _clock = clock::auto_inc().unwrap();
    clock::fetch_add(Duration::from_nanos(1));
    thread::spawn(move || {
        assert_clock_eq!(Duration::from_nanos(1));
    })
    .join()
    .unwrap();
}

#[test]
fn auto_inc_thread_sleep() {
    let _clock = clock::auto_inc().unwrap();
    thread::sleep(Duration::from_nanos(1));
    assert_clock_eq!(Duration::from_nanos(1));
}

#[test]
fn auto_inc_is_not_global() {
    let _clock = clock::auto_inc().unwrap();
    // Don't use mock thread::spawn here!
    std::thread::spawn(move || {
        thread::sleep(Duration::from_nanos(1));
    })
    .join()
    .unwrap();
    assert_clock_eq!(Duration::from_nanos(0));
}

#[test]
fn auto_inc_thread_join_sync() {
    let _clock = clock::auto_inc().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_nanos(1));
    })
    .join()
    .unwrap();
    assert_clock_eq!(Duration::from_nanos(1));
}
