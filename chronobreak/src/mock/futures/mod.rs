/// **Mock** of [`futures::executor`](https://docs.rs/futures/0.3.6/futures/executor/index.html)
pub mod executor;

pub use futures::{
    channel, future, io, join, lock, never, pending, pin_mut, poll, prelude, ready, select,
    select_biased, sink, stream, task, try_join,
};
