use crate::util;
use futures::future::FutureObj;
use futures::task::Spawn;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};

#[chronobreak]
mod mock {
    pub use futures::executor::*;
    pub use std::time::*;
}
use mock::*;

#[test]
#[ignore]
fn traits() {
    util::is_clone::<ThreadPool>();
    util::is_debug::<ThreadPool>();
    util::is_send::<ThreadPool>();
    util::is_sync::<ThreadPool>();

    util::is_default::<ThreadPoolBuilder>();
    util::is_debug::<ThreadPoolBuilder>();
    util::is_send::<ThreadPoolBuilder>();
    util::is_sync::<ThreadPoolBuilder>();
}

#[chronobreak::test]
fn spawn_ok() {
    let pool = ThreadPool::new().unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let barrier2 = barrier.clone();
    pool.spawn_ok(async move {
        clock::advance(Duration::from_nanos(2));
        barrier2.wait();
    });
    barrier.wait();
}

#[chronobreak::test]
fn spawn_obj() {
    let pool = ThreadPool::new().unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let barrier2 = barrier.clone();
    pool.spawn_obj(FutureObj::new(Box::pin(async move {
        clock::advance(Duration::from_nanos(2));
        barrier2.wait();
    })))
    .unwrap();
    barrier.wait();
}

#[chronobreak::test]
fn spawn_obj_ok() {
    let pool = ThreadPool::new().unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let barrier2 = barrier.clone();
    pool.spawn_obj_ok(FutureObj::new(Box::pin(async move {
        clock::advance(Duration::from_nanos(2));
        barrier2.wait();
    })));
    barrier.wait();
}

#[chronobreak::test]
fn with_custom_after_start_is_mocked() {
    let executed = Arc::new(AtomicBool::default());
    let executed2 = executed.clone();
    let pool = ThreadPool::builder()
        .after_start(move |_| executed2.store(true, Ordering::Relaxed))
        .create()
        .unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let barrier2 = barrier.clone();
    pool.spawn_ok(async move {
        clock::advance(Duration::from_nanos(2));
        barrier2.wait();
    });
    barrier.wait();
    assert_eq! {executed.load(Ordering::Relaxed), true};
}
