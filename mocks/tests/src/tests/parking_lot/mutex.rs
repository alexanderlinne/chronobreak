use chronobreak::clock;
use std::sync::{Arc, Barrier};

#[chronobreak]
mod mock {
    pub use parking_lot::*;
    pub use std::thread;
    pub use std::time::*;
}
use mock::*;

#[chronobreak::test]
fn lock_sycs_participants() {
    test_impl()
}

#[chronobreak::test(frozen)]
fn lock_doesnt_freeze() {
    test_impl()
}

fn test_impl() {
    let start_time = Instant::now();
    let data = Arc::new((Mutex::new(()), Barrier::new(2)));
    let data2 = data.clone();
    thread::spawn(move || {
        clock::advance(Duration::from_millis(1));
        data2.0.lock();
        data2.1.wait();
    });
    data.1.wait();
    data.0.lock();
    assert_eq! {Instant::now(), start_time + Duration::from_millis(1)}
}

#[chronobreak::test]
fn guard_impls_deref() {
    let mutex = Mutex::new(true);
    assert_eq! {*mutex.lock(), true};
}

#[chronobreak::test]
fn guard_impls_deref_mut() {
    let mutex = Mutex::new(true);
    *mutex.lock() = false;
}
