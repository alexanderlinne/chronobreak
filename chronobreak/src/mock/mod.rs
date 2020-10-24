/// Mocks for [`futures`](https://crates.io/crates/futures) 0.3
#[cfg(feature = "futures")]
pub mod futures;

/// Mocks for [`futures-timer`](https://crates.io/crates/futures-timer) 3.0
#[cfg(feature = "futures-timer")]
pub mod futures_timer;

/// Mocks for [`parking_lot`](https://crates.io/crates/parking_lot) 0.11
#[cfg(feature = "parking_lot")]
pub mod parking_lot;

/// Mocks for [the standard library](https://doc.rust-lang.org/std/index.html)
pub mod std;
