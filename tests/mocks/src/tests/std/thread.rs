#[chronobreak]
mod mock {
    pub use std::sync::Arc;
    pub use std::thread;
    pub use std::time::*;
}
use mock::*;

#[chronobreak::test]
fn mocked_transfers_on_spawn() {
    clock::advance(Duration::from_nanos(1));
    thread::spawn(move || {
        assert_clock_eq!(Duration::from_nanos(1));
    })
    .join()
    .unwrap();
}

#[chronobreak::test]
fn sleep_advances() {
    thread::sleep(Duration::from_nanos(1));
    assert_clock_eq!(Duration::from_nanos(1));
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
    thread::spawn(move || {
        thread::sleep(Duration::from_nanos(1));
    })
    .join()
    .unwrap();
    assert_clock_eq!(Duration::from_nanos(1));
}

#[test]
fn join_handle_works_with_non_mocked_clock() {
    thread::spawn(|| {}).join().unwrap();
}
