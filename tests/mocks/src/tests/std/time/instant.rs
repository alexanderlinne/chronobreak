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

#[chronobreak::test]
fn elapsed() {
    let start = Instant::now();
    clock::advance(Duration::from_secs(1));
    assert_eq! {start.elapsed(), Duration::from_secs(1)};
}

#[chronobreak::test]
fn checked_add() {
    let dur = Duration::from_secs(1);
    let instant = Instant::now().checked_add(dur);
    assert_eq! {instant, Some(Instant::Mocked(dur))};
}

#[chronobreak::test]
fn checked_add_secs_overflow() {
    let dur = Duration::from_secs(u64::MAX);
    let instant = Instant::now().checked_add(dur);
    assert_eq! {instant, Some(Instant::Mocked(dur))};
    let instant = instant.unwrap().checked_add(Duration::from_secs(1));
    assert_eq! {instant, None};
}

#[chronobreak::test]
fn checked_add_nanos_overflow() {
    let dur = Duration::from_secs(u64::MAX)
        + Duration::from_nanos(Duration::from_secs(1).as_nanos() as u64 - 1);
    let instant = Instant::now().checked_add(dur);
    assert_eq! {instant, Some(Instant::Mocked(dur))};
    let instant = instant.unwrap().checked_add(Duration::from_nanos(1));
    assert_eq! {instant, None};
}

#[chronobreak::test]
fn checked_sub() {
    let dur = Duration::from_secs(1);
    clock::advance(dur);
    let instant = Instant::now().checked_sub(dur);
    assert_eq! {instant, Some(Instant::Mocked(Duration::default()))};
}

#[chronobreak::test]
fn checked_sub_underflow() {
    let instant = Instant::now().checked_sub(Duration::from_secs(1));
    assert_eq! {instant, None};
    let instant = Instant::now().checked_sub(Duration::from_nanos(1));
    assert_eq! {instant, None};
}
