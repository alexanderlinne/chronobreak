use crate::clock;
use crate::mock::std::time;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Delay {
    delay: futures_timer::Delay,
    dur: time::Duration,
}

impl Delay {
    pub fn new(dur: time::Duration) -> Self {
        Self {
            delay: futures_timer::Delay::new(dur),
            dur,
        }
    }

    pub fn reset(&mut self, dur: time::Duration) {
        self.delay.reset(dur);
        self.dur = dur;
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match_clock_strategy! {
            Sys => {
                use futures::future::FutureExt;
                self.delay.poll_unpin(cx)
            },
            Manual => {
                unimplemented! {}
            },
            AutoInc => {
                clock::fetch_add(self.dur);
                Poll::Ready(())
            },
        }
    }
}
