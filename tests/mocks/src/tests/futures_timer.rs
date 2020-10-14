use chronobreak::clock;
#[chronobreak]
use futures_timer::*;
#[chronobreak]
use std::time::*;

#[async_std::test]
async fn delay_increases_auto_inc() {
    let _clock = clock::auto_inc().unwrap();
    Delay::new(Duration::from_nanos(1)).await;
    assert_clock_eq!(Duration::from_nanos(1));
}

#[async_std::test]
async fn delay_auto_inc_reset() {
    let _clock = clock::auto_inc().unwrap();
    let mut delay = Delay::new(Duration::from_nanos(10));
    delay.reset(Duration::from_nanos(1));
    delay.await;
    assert_clock_eq!(Duration::from_nanos(1));
}

#[async_std::test]
async fn delay_auto_inc_saves_timeout_on_construction() {
    let _clock = clock::auto_inc().unwrap();
    let delay1 = Delay::new(Duration::from_nanos(1));
    let delay2 = Delay::new(Duration::from_nanos(1));
    delay1.await;
    delay2.await;
    assert_clock_eq!(Duration::from_nanos(1));
}
