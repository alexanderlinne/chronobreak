#[cfg(feature = "crossbeam")]
pub mod crossbeam;

#[cfg(feature = "futures-timer")]
pub mod futures_timer;

#[cfg(feature = "parking_lot")]
pub mod parking_lot;

pub mod std;
