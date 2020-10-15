#[chronobreak]
use std::thread;
#[chronobreak]
use std::time::*;

#[chronobreak::test(frozen)]
fn main_thread_is_registered() {
    let main_thread = thread::current();
    thread::spawn(move || {
        clock::unfreeze();
        main_thread.expect_blocking_wait();
        clock::advance(Duration::from_millis(1))
    });
    clock::advance(Duration::from_millis(1));
}
