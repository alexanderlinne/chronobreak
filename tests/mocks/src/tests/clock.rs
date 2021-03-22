use chronobreak::clock;
#[chronobreak]
use std::thread;
#[chronobreak]
use std::time::*;

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

#[chronobreak::test]
fn mock_is_not_global() {
    let start_time = Instant::now();

    // Tests that the mocked clock is global for only the threads that have been
    // created within the current test. This is required as multiple tests may
    // run in parallel.

    // Don't use mock thread::spawn here!
    std::thread::spawn(move || {
        thread::sleep(Duration::from_nanos(1));
    })
    .join()
    .unwrap();
    assert_eq! {Instant::now(), start_time};
}
