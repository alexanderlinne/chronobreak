#[chronobreak]
use std::thread;
#[chronobreak]
use std::time::*;

#[chronobreak::test]
fn mock_is_not_global() {
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
