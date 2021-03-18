//! [![github]](https://github.com/alexanderlinne/chronobreak)&ensp;[![crates-io]](https://crates.io/crates/chronobreak)&ensp;[![docs-rs]](https://docs.rs/chronobreak)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K
//!
//! # chronobreak: Rust mocks for deterministic time testing
//!
//! chronobreak is a library of test mocks for deterministically testing
//! any time-based property of a given test subject.
//!
//! # Motivation
//!
//! Let's say we've written a simple function that returns some value after
//! a given timepoint is reached:
//!
//! ```
//! use std::time::*;
//! use std::thread;
//!
//! fn return_at<T>(time: Instant, t: T) -> T {
//!     if Instant::now() < time {
//!         thread::sleep(time.saturating_duration_since(Instant::now()));
//!     }
//!     t
//! }
//! ```
//!
//! We now may want to test whether this function actually sleeps as expected:
//!
//! ```
//! #[test]
//! fn test_return_at() {
//!     let return_time = Instant::now() + Duration::from_secs(1);
//!     return_at(return_time, 0);
//!     assert_eq! {Instant::now(), return_time};
//! }
//! ```
//!
//! This test case will most certainly fail. One common strategy to resolve this
//! issue is to expect the time to be within some interval instead of comparing
//! for exact equality. But this will never guarantee that test cases
//! similar to the above will succeed deterministically.
//!
//! # chronobreak to the rescue
//!
//! So how can we deterministically pass the test?
//!
//! First, for the mocked clock to work as expected, it is important that for every
//! import for which chronobreak provides a mock, the mock is used when compiling
//! tests:
//!
//! ```
//! # #[macro_use]
//! # extern crate chronobreak;
//! #[chronobreak]
//! use std::time::*; // will be replaced with `use chronobreak::mock::std::time::*; for tests
//! #[chronobreak]
//! use std::thread;
//! # fn main() {}
//! ```
//!
//! To make it as easy as possible to not accidentally miss any mock,
//! chronobreak also re-exports all items for the supported libraries that do
//! not require to be mocked.
//!
//! Now we can test with a mocked clock by simply exchanging `#[test]` with
//! `#[chronobreak::test]`:
//!
//! ```
//! #[chronobreak::test]
//! fn test_return_at() {
//!     let return_time = Instant::now() + Duration::from_secs(1);
//!     return_at(return_time, 0);
//!     assert_eq! {Instant::now(), return_time};
//! }
//! ```
//!
//! What happens here is that the mocked version of `thread::sleep` will
//! act as a decorator for the original function. If the clock is mocked
//! for the current test case, it will advance it by exactly one second.
//! If it is not mocked, `thread::sleep` will directly delegate to the
//! original function.
//!
//! # The frozen clock
//!
//! In addition to it's default behaviour of automatically advancing the
//! clock for any timed wait, chronobreak allows freezing the clock. This
//! causes all timed waits to instead block until some other thread advances
//! the clock either manually through [`clock::advance`](clock/fn.advance.html)
//! or [`clock::advance_to`](clock/fn.advance_to.html) or by performing a
//! timed wait while not being frozen.
//!
//! This feature is mainly intended to be used in combination with the
//! `extended-apis` feature which adds
//! [`Thread::expect_timed_wait`](mock/std/thread/struct.Thread.html#method.expect_timed_wait)
//! and
//! [`JoinHandle::expect_timed_wait`](mock/std/thread/struct.JoinHandle.html#method.expect_timed_wait)
//! to the public APIs of the mocked versions of those classes. Those functions
//! make it possible to wait for another thread to enter a timed wait before
//! resuming. This is useful for situations where e.g. the test subject is
//! a concurrent data structure and it must be tested that it behaves correctly
//! when it receives input from one thread while it already entered a timed
//! wait on another thread.
//!

extern crate chronobreak_derive;

/// Exported macros.
#[macro_use]
mod macros;

/// Types that represent errors that may occur while using chronobreak.
pub mod error;

/// The mocked clock.
pub mod clock;
/// The shared clock.
mod shared_clock;

/// Mocks for the standard library and several popular crates.
pub mod mock;

pub use chronobreak_derive::chronobreak;
pub use chronobreak_derive::test;
