use chronobreak::clock;
#[chronobreak]
use std::thread;
#[chronobreak]
use std::time::*;

#[test]
fn main_thread_is_registered() {
    let _clock = clock::mocked();
    let main_thread = thread::current();
    thread::spawn(move || {
        clock::unfreeze();
        main_thread.expect_blocking_wait();
        clock::advance(Duration::from_millis(1))
    });
    clock::freeze();
    clock::advance(Duration::from_millis(1));
}
