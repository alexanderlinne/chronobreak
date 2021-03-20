#[macro_use]
extern crate chronobreak_derive;

pub use chronobreak::{clock, mock};

fn main() {
    apply!(0);
}
