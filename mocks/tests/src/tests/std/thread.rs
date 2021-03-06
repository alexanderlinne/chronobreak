#[chronobreak]
mod mock {
    pub use std::sync::Arc;
    pub use std::thread;
    pub use std::time::*;
}
use mock::*;

#[chronobreak::test]
fn mocked_transfers_on_spawn() {
    let start_time = Instant::now();
    clock::advance(Duration::from_nanos(1));
    thread::spawn(move || {
        assert_eq!(Instant::now(), start_time + Duration::from_nanos(1));
    })
    .join()
    .unwrap();
}

#[chronobreak::test]
fn sleep_advances() {
    let start_time = Instant::now();
    thread::sleep(Duration::from_nanos(1));
    assert_eq!(Instant::now(), start_time + Duration::from_nanos(1));
}

#[chronobreak::test]
fn mocked_thread_join_syncs() {
    mocked_thread_join_syncs_impl()
}

#[chronobreak::test(frozen)]
fn join_doesnt_freeze() {
    mocked_thread_join_syncs_impl()
}

fn mocked_thread_join_syncs_impl() {
    let start_time = Instant::now();
    thread::spawn(move || {
        thread::sleep(Duration::from_nanos(1));
    })
    .join()
    .unwrap();
    assert_eq!(Instant::now(), start_time + Duration::from_nanos(1));
}

#[test]
fn join_handle_works_with_non_mocked_clock() {
    thread::spawn(|| {}).join().unwrap();
}
