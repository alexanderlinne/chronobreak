use crate::clock;
use crate::mock::std::time::*;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// **Mock** of [`futures_timer::Delay`](https://docs.rs/futures-timer/3.0.2/futures_timer/struct.Delay.html)
pub struct Delay {
    delay: futures_timer::Delay,
    mocked: clock::DelayFuture,
}

impl Delay {
    pub fn new(dur: Duration) -> Self {
        Self {
            delay: futures_timer::Delay::new(dur),
            mocked: clock::DelayFuture::new(dur),
        }
    }

    pub fn reset(&mut self, dur: Duration) {
        if clock::is_mocked() {
            self.mocked.reset(dur)
        } else {
            self.delay.reset(dur)
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if clock::is_mocked() {
            unsafe { self.map_unchecked_mut(|this| &mut this.mocked) }.poll(cx)
        } else {
            unsafe { self.map_unchecked_mut(|this| &mut this.delay) }.poll(cx)
        }
    }
}
