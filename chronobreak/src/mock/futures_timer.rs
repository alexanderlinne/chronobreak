use crate::clock;
use crate::mock::std::time::Duration;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// **Mock** of [`futures_timer::Delay`](https://docs.rs/futures-timer/3.0.2/futures_timer/struct.Delay.html)
#[pin_project(project = DelayProj)]
pub enum Delay {
    Actual(#[pin] futures_timer::Delay),
    Mocked(#[pin] clock::DelayFuture),
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
            Self::Actual(delay) => delay.reset(dur),
            Self::Mocked(delay) => delay.reset(dur),
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            DelayProj::Actual(delay) => delay.poll(cx),
            DelayProj::Mocked(delay) => delay.poll(cx),
        }
    }
}
