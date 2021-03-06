mod delay {
    use futures::task::{waker, ArcWake, Context, Poll};
    #[chronobreak]
    use futures_timer::*;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Barrier};
    #[chronobreak]
    use std::thread;
    #[chronobreak]
    use std::time::*;

    #[derive(Default)]
    struct BooleanWaker {
        woken: AtomicBool,
    }

    impl ArcWake for BooleanWaker {
        fn wake_by_ref(this: &Arc<Self>) {
            this.woken.store(true, Ordering::Relaxed);
        }
    }

    #[chronobreak::test]
    async fn increases_auto_inc() {
        let start_time = Instant::now();
        Delay::new(Duration::from_nanos(1)).await;
        assert_eq! {Instant::now(), start_time + Duration::from_nanos(1)};
    }

    #[chronobreak::test]
    async fn auto_inc_reset() {
        let start_time = Instant::now();
        let mut delay = Delay::new(Duration::from_nanos(10));
        delay.reset(Duration::from_nanos(1));
        delay.await;
        assert_eq! {Instant::now(), start_time + Duration::from_nanos(1)};
    }

    #[chronobreak::test]
    async fn auto_inc_saves_timeout_on_construction() {
        let start_time = Instant::now();
        let delay1 = Delay::new(Duration::from_nanos(1));
        let delay2 = Delay::new(Duration::from_nanos(1));
        delay1.await;
        delay2.await;
        assert_eq! {Instant::now(), start_time + Duration::from_nanos(1)};
    }

    #[chronobreak::test(frozen)]
    async fn frozen_poll() {
        let start_time = Instant::now();
        let barrier = Arc::new(Barrier::new(2));
        let barrier2 = barrier.clone();
        thread::spawn(move || {
            barrier.wait();
            clock::advance(Duration::from_nanos(1));
            barrier.wait();
            barrier.wait();
            clock::advance(Duration::from_nanos(1));
            barrier.wait();
        });
        use futures::Future;
        let mut delay = Delay::new(Duration::from_nanos(2));
        let boolean_waker = Arc::new(BooleanWaker::default());
        let waker = waker(boolean_waker.clone());
        let mut context = Context::from_waker(&waker);
        matches! { unsafe { Pin::new_unchecked(&mut delay) }.poll(&mut context), Poll::Pending };
        assert_eq! {Instant::now(), start_time + Duration::default()};
        assert_eq! {boolean_waker.woken.load(Ordering::Relaxed), false};
        barrier2.wait();
        barrier2.wait();
        matches! { unsafe { Pin::new_unchecked(&mut delay) }.poll(&mut context), Poll::Pending };
        assert_eq! {Instant::now(), start_time + Duration::from_nanos(1)};
        assert_eq! {boolean_waker.woken.load(Ordering::Relaxed), false};
        barrier2.wait();
        barrier2.wait();
        matches! { unsafe { Pin::new_unchecked(&mut delay) }.poll(&mut context), Poll::Ready(()) };
        assert_eq! {Instant::now(), start_time + Duration::from_nanos(2)};
        assert_eq! {boolean_waker.woken.load(Ordering::Relaxed), true};
    }

    #[chronobreak::test(frozen)]
    fn frozen_delay_is_blocking() {
        let main_thread = thread::current();
        let thread = thread::spawn(move || {
            main_thread.expect_timed_wait();
            clock::advance(Duration::from_nanos(1));
        });
        futures::executor::block_on(Delay::new(Duration::from_nanos(1)));
        thread.join().unwrap();
    }
}
