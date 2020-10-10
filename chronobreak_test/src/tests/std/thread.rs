use chronobreak::clock::*;
#[chronobreak]
use std::sync::Arc;
use std::sync::Barrier;
#[chronobreak]
use std::thread;
#[chronobreak]
use std::time::*;

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
