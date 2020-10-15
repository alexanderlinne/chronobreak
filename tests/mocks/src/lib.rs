#[allow(unused_imports)]
#[macro_use(chronobreak, assert_clock_eq)]
extern crate chronobreak;

#[cfg(test)]
#[macro_use]
extern crate paste;

#[cfg(test)]
#[macro_use]
mod util;

#[cfg(test)]
mod tests;
