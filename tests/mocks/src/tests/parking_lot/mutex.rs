use chronobreak::clock;
use std::sync::{Arc, Barrier};

#[chronobreak]
mod mock {
    pub use parking_lot::*;
    pub use std::thread;
    pub use std::time::*;
}
use mock::*;

impl_debug! {condvar, Condvar::new()}
impl_default! {condvar, Condvar}

#[chronobreak::test]
fn lock_sycs_participants() {
    test_impl()
}

#[chronobreak::test(frozen)]
fn lock_doesnt_freeze() {
    test_impl()
}

fn test_impl() {
    let data = Arc::new((Mutex::new(()), Barrier::new(2)));
    let data2 = data.clone();
    thread::spawn(move || {
        clock::unfreeze();
        clock::advance(Duration::from_millis(1));
        data2.0.lock();
        data2.1.wait();
    });
    data.1.wait();
    data.0.lock();
    assert_clock_eq! {Duration::from_millis(1)}
}
