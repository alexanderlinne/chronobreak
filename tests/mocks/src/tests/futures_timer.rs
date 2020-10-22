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

    #[chronobreak::test]
    async fn frozen_poll() {
        let barrier = Arc::new(Barrier::new(2));
        let barrier2 = barrier.clone();
        thread::spawn(move || {
            use futures::Future;
            clock::freeze();
            let mut delay = Delay::new(Duration::from_nanos(2));
            let waker = waker(Arc::new(BooleanWaker::default()));
            let mut context = Context::from_waker(&waker);
            matches! { unsafe { Pin::new_unchecked(&mut delay) }.poll(&mut context), Poll::Pending };
            assert_clock_eq! {Duration::default()};
            barrier2.wait();
            barrier2.wait();
            matches! { unsafe { Pin::new_unchecked(&mut delay) }.poll(&mut context), Poll::Pending };
            assert_clock_eq! {Duration::from_nanos(1)};
            barrier2.wait();
            barrier2.wait();
            matches! { unsafe { Pin::new_unchecked(&mut delay) }.poll(&mut context), Poll::Ready(()) };
            assert_clock_eq! {Duration::from_nanos(2)};
        });
        barrier.wait();
        clock::advance(Duration::from_nanos(1));
        barrier.wait();
        barrier.wait();
        clock::advance(Duration::from_nanos(1));
        barrier.wait();
    }
}
