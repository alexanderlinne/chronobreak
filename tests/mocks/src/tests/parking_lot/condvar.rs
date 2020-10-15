use chronobreak::clock;

#[chronobreak]
mod mock {
    pub use parking_lot::*;
}
use mock::*;

impl_debug! {condvar, Condvar::new()}
impl_default! {condvar, Condvar}

#[test]
fn spawn_ok() {
    let _clock = clock::mocked().unwrap();
}
