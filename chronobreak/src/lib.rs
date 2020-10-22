extern crate chronobreak_derive;

#[macro_use]
mod macros;

/// The mocked clock.
pub mod clock;
mod shared_clock;

/// Mocks for the standard library and several popular crates.
pub mod mock;

pub use chronobreak_derive::chronobreak;
pub use chronobreak_derive::test;
