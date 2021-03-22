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
fn wait_sycs_with_notifier() {
    wait_sycs_with_notifier_impl()
}

#[chronobreak::test(frozen)]
fn wait_doesnt_freeze() {
    wait_sycs_with_notifier_impl()
}

fn wait_sycs_with_notifier_impl() {
    let start_time = Instant::now();
    let data = Arc::new((Mutex::new(()), Condvar::new(), Barrier::new(2)));
    let data2 = data.clone();
    thread::spawn(move || {
        clock::advance(Duration::from_millis(1));
        data2.2.wait();
        data2.0.lock();
        data2.1.notify_all();
    });
    let mut lock = data.0.lock();
    data.2.wait();
    data.1.wait(&mut lock);
    assert_eq! {Instant::now(), start_time + Duration::from_millis(1)}
}

#[chronobreak::test]
fn advances_notifier() {
    advances_notifier_impl()
}

#[chronobreak::test(frozen)]
fn notify_doesnt_freeze() {
    advances_notifier_impl()
}

fn advances_notifier_impl() {
    let start_time = Instant::now();
    let data = Arc::new((Mutex::new(()), Condvar::new(), Barrier::new(2)));
    let data2 = data.clone();
    thread::spawn(move || {
        let mut lock = data.0.lock();
        clock::advance(Duration::from_millis(1));
        data.2.wait();
        data.1.wait(&mut lock);
    });
    data2.2.wait();
    data2.0.lock();
    data2.1.notify_all();
    assert_eq! {Instant::now(), start_time + Duration::from_millis(1)}
}
