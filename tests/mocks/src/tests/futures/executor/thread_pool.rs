use chronobreak::clock;
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

impl_clone! {thread_pool, ThreadPool::new().unwrap()}
impl_debug! {thread_pool, ThreadPool::new().unwrap()}
impl_default! {thread_pool_builder, ThreadPoolBuilder}
impl_debug! {thread_pool_builder, ThreadPool::builder()}

#[test]
fn spawn_ok() {
    let _clock = clock::mocked().unwrap();
    let pool = ThreadPool::new().unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let barrier2 = barrier.clone();
    pool.spawn_ok(async move {
        clock::advance(Duration::from_nanos(2));
        barrier2.wait();
    });
    barrier.wait();
}

#[test]
fn spawn_obj() {
    let _clock = clock::mocked().unwrap();
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

#[test]
fn spawn_obj_ok() {
    let _clock = clock::mocked().unwrap();
    let pool = ThreadPool::new().unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let barrier2 = barrier.clone();
    pool.spawn_obj_ok(FutureObj::new(Box::pin(async move {
        clock::advance(Duration::from_nanos(2));
        barrier2.wait();
    })));
    barrier.wait();
}

#[test]
fn with_custom_after_start_is_mocked() {
    let _clock = clock::mocked().unwrap();
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
