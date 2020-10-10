use async_std::task;
use chronobreak::clock;
#[chronobreak]
use futures_timer::*;
#[chronobreak]
use std::time::*;

#[test]
fn delay_increases_auto_inc() {
    let _clock = clock::auto_inc().unwrap();
    task::block_on(async {
        Delay::new(Duration::from_nanos(1)).await;
    });
    assert_clock_eq!(Duration::from_nanos(1));
}

#[test]
fn delay_auto_inc_reset() {
    let _clock = clock::auto_inc().unwrap();
    task::block_on(async {
        let mut delay = Delay::new(Duration::from_nanos(10));
        delay.reset(Duration::from_nanos(1));
        delay.await;
    });
    assert_clock_eq!(Duration::from_nanos(1));
}
