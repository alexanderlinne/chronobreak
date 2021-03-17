use chronobreak::clock;
#[chronobreak]
use std::time::*;

#[test]
#[should_panic]
fn duration_since_incompatible_1() {
    let lhs = Instant::now();
    let _clock = clock::mock().unwrap();
    let rhs = Instant::now();
    let _ = lhs.duration_since(rhs);
}

#[test]
#[should_panic]
fn duration_since_incompatible_2() {
    let lhs = Instant::now();
    let _clock = clock::mock().unwrap();
    let rhs = Instant::now();
    let _ = rhs.duration_since(lhs);
}

#[test]
#[should_panic]
fn checked_duration_since_incompatible_1() {
    let lhs = Instant::now();
    let _clock = clock::mock().unwrap();
    let rhs = Instant::now();
    let _ = lhs.checked_duration_since(rhs);
}

#[test]
#[should_panic]
fn checked_duration_since_incompatible_2() {
    let lhs = Instant::now();
    let _clock = clock::mock().unwrap();
    let rhs = Instant::now();
    let _ = rhs.checked_duration_since(lhs);
}

#[test]
#[should_panic]
fn saturating_duration_since_incompatible_1() {
    let lhs = Instant::now();
    let _clock = clock::mock().unwrap();
    let rhs = Instant::now();
    let _ = lhs.duration_since(rhs);
}

#[test]
#[should_panic]
fn saturating_duration_since_incompatible_2() {
    let lhs = Instant::now();
    let _clock = clock::mock().unwrap();
    let rhs = Instant::now();
    let _ = rhs.duration_since(lhs);
}

#[test]
#[should_panic]
fn cmp_incompatible_1() {
    let lhs = Instant::now();
    let _clock = clock::mock().unwrap();
    let rhs = Instant::now();
    let _ = lhs.cmp(&rhs);
}

#[test]
#[should_panic]
fn cmp_incompatible_2() {
    let lhs = Instant::now();
    let _clock = clock::mock().unwrap();
    let rhs = Instant::now();
    let _ = rhs.cmp(&lhs);
}

#[test]
#[should_panic]
fn partial_cmp_incompatible_1() {
    let lhs = Instant::now();
    let _clock = clock::mock().unwrap();
    let rhs = Instant::now();
    let _ = lhs.partial_cmp(&rhs);
}

#[test]
#[should_panic]
fn partial_cmp_incompatible_2() {
    let lhs = Instant::now();
    let _clock = clock::mock().unwrap();
    let rhs = Instant::now();
    let _ = rhs.partial_cmp(&lhs);
}

#[test]
#[should_panic]
fn eq_incompatible_1() {
    let lhs = Instant::now();
    let _clock = clock::mock().unwrap();
    let rhs = Instant::now();
    let _ = lhs.eq(&rhs);
}

#[test]
#[should_panic]
fn eq_incompatible_2() {
    let lhs = Instant::now();
    let _clock = clock::mock().unwrap();
    let rhs = Instant::now();
    let _ = rhs.eq(&lhs);
}
