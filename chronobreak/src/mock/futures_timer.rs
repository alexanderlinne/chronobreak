use crate::clock;
use crate::mock::std::time::*;
use crate::shared_clock::TimedWakerHandle;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// **Mock** of [`futures_timer::Delay`](https://docs.rs/futures-timer/3.0.2/futures_timer/struct.Delay.html)
pub enum Delay {
    Actual(futures_timer::Delay),
    Mocked((Instant, Option<TimedWakerHandle>)),
}

impl Delay {
    pub fn new(dur: Duration) -> Self {
        if clock::is_mocked() {
            Self::Mocked((Instant::now() + dur, None))
        } else {
            Self::Actual(futures_timer::Delay::new(dur))
        }
    }

    pub fn reset(&mut self, dur: Duration) {
        match self {
            Self::Mocked((timeout, waker_handle)) => {
                *timeout = Instant::now() + dur;
                if let Some(handle) = waker_handle.take() {
                    *waker_handle = clock::register_timed_waker(handle.waker(), *timeout);
                }
            }
            Self::Actual(delay) => delay.reset(dur),
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::into_inner(self) {
            Self::Mocked((timeout, waker_handle)) => {
                if clock::is_frozen() {
                    let handle = clock::register_timed_waker(cx.waker().clone(), *timeout);
                    if let Some(handle) = handle {
                        *waker_handle = Some(handle);
                        Poll::Pending
                    } else {
                        *waker_handle = None;
                        Poll::Ready(())
                    }
                } else {
                    clock::advance_to(*timeout);
                    Poll::Ready(())
                }
            }
            Self::Actual(delay) => unsafe { Pin::new_unchecked(delay).poll(cx) },
        }
    }
}
