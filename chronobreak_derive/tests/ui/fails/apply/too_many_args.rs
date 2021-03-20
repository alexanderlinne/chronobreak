#[macro_use]
extern crate chronobreak_derive;

pub use chronobreak::{clock, mock};

fn main() {
    apply!(0, |_| 0, |_| 0, 0);
}
