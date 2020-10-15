#[chronobreak]
mod mock {
    pub use std::sync::Arc;
    pub use std::thread;
    pub use std::time::*;
}
use mock::*;

#[chronobreak::test]
fn mocked_transfers_on_thread_spawn() {
    clock::advance(Duration::from_nanos(1));
    thread::spawn(move || {
        assert_clock_eq!(Duration::from_nanos(1));
    })
    .join()
    .unwrap();
}

#[chronobreak::test]
fn mocked_thread_sleep() {
    thread::sleep(Duration::from_nanos(1));
    assert_clock_eq!(Duration::from_nanos(1));
}

#[chronobreak::test]
fn mocked_is_not_global() {
    // Tests that the mocked clock is global for only the threads that have been
    // created within the current test. This is required as multiple tests may
    // run in parallel.

    // Don't use mock thread::spawn here!
    std::thread::spawn(move || {
        thread::sleep(Duration::from_nanos(1));
    })
    .join()
    .unwrap();
    assert_clock_eq!(Duration::from_nanos(0));
}

#[chronobreak::test]
fn mocked_thread_join_sync() {
    thread::spawn(move || {
        thread::sleep(Duration::from_nanos(1));
    })
    .join()
    .unwrap();
    assert_clock_eq!(Duration::from_nanos(1));
}

#[chronobreak::test]
fn frozen_wait_is_blocking() {
    let thread = thread::spawn(move || {
        clock::freeze();
        clock::advance(Duration::from_nanos(1));
    });
    thread.expect_blocking_wait();
    clock::advance(Duration::from_nanos(1));
    thread.join().unwrap();
}

#[test]
fn join_handle_works_with_non_mocked_clock() {
    thread::spawn(|| {}).join().unwrap();
}
