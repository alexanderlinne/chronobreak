use crate::clock;
use crate::mock::std::time::*;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// **Mock** of [`futures_timer::Delay`](https://docs.rs/futures-timer/3.0.2/futures_timer/struct.Delay.html)
pub enum Delay {
    Actual(futures_timer::Delay),
    Mocked(clock::DelayFuture),
}

impl Delay {
    pub fn new(dur: Duration) -> Self {
        if clock::is_mocked() {
            Self::Mocked(clock::DelayFuture::new(dur))
        } else {
            Self::Actual(futures_timer::Delay::new(dur))
        }
    }

    pub fn reset(&mut self, dur: Duration) {
        match self {
            Self::Mocked(fut) => fut.reset(dur),
            Self::Actual(delay) => delay.reset(dur),
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::into_inner(self) {
            Self::Mocked(fut) => unsafe { Pin::new_unchecked(fut) }.poll(cx),
            Self::Actual(delay) => unsafe { Pin::new_unchecked(delay) }.poll(cx),
        }
    }
}
