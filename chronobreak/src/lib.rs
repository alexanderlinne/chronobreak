extern crate chronobreak_derive;

#[macro_use]
pub mod clock;
pub mod mock;

pub use chronobreak_derive::chronobreak;
pub use chronobreak_derive::test;
