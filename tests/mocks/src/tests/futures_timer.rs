mod delay {
    #[chronobreak]
    use futures_timer::*;
    #[chronobreak]
    use std::time::*;

    #[chronobreak::test]
    async fn increases_auto_inc() {
        Delay::new(Duration::from_nanos(1)).await;
        assert_clock_eq!(Duration::from_nanos(1));
    }

    #[chronobreak::test]
    async fn auto_inc_reset() {
        let mut delay = Delay::new(Duration::from_nanos(10));
        delay.reset(Duration::from_nanos(1));
        delay.await;
        assert_clock_eq!(Duration::from_nanos(1));
    }

    #[chronobreak::test]
    async fn auto_inc_saves_timeout_on_construction() {
        let delay1 = Delay::new(Duration::from_nanos(1));
        let delay2 = Delay::new(Duration::from_nanos(1));
        delay1.await;
        delay2.await;
        assert_clock_eq!(Duration::from_nanos(1));
    }
}
