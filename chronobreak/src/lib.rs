extern crate chronobreak_derive;

/// The mocked clock.
#[macro_use]
pub mod clock;
/// Mocks for the standard library and several popular crates.
pub mod mock;

pub use chronobreak_derive::chronobreak;
pub use chronobreak_derive::test;
