use crate::clock;
use crate::mock::std::time::*;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// **Mock** of [`futures_timer::Delay`](https://docs.rs/futures-timer/3.0.2/futures_timer/struct.Delay.html)
pub struct Delay {
    delay: futures_timer::Delay,
    timeout: Instant,
}

impl Delay {
    pub fn new(dur: Duration) -> Self {
        Self {
            delay: futures_timer::Delay::new(dur),
            timeout: Instant::now() + dur,
        }
    }

    pub fn reset(&mut self, dur: Duration) {
        self.delay.reset(dur);
        self.timeout = Instant::now() + dur;
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if clock::is_mocked() {
            clock::advance(self.timeout.saturating_duration_since(Instant::now()));
            Poll::Ready(())
        } else {
            use futures::future::FutureExt;
            self.delay.poll_unpin(cx)
        }
    }
}
