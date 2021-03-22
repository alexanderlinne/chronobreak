use chronobreak::clock;
#[chronobreak]
use std::time::*;

#[test]
#[should_panic]
fn duration_since_incompatible_1() {
    let lhs = SystemTime::now();
    let _clock = clock::mock().unwrap();
    let rhs = SystemTime::now();
    let _ = lhs.duration_since(rhs);
}

#[test]
#[should_panic]
fn duration_since_incompatible_2() {
    let lhs = SystemTime::now();
    let _clock = clock::mock().unwrap();
    let rhs = SystemTime::now();
    let _ = rhs.duration_since(lhs);
}

#[chronobreak::test]
fn duration_since_unix_epoch() {
    assert_eq! {SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap(), Duration::default()};
}

#[chronobreak::test]
fn elapsed() {
    let start = SystemTime::now();
    clock::advance(Duration::from_secs(1));
    assert_eq! {start.elapsed().unwrap(), Duration::from_secs(1)};
}

#[chronobreak::test]
fn elapsed_unix_epoch() {
    assert_eq! {SystemTime::UNIX_EPOCH.elapsed().unwrap(), Duration::default()};
}
