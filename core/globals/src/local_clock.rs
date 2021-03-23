use crate::shared_clock::{SharedClock, Timepoint};
use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    /// State of the mocked clock. None if the clock is not mocked.
    pub static STATE: RefCell<Option<LocalClock>> = RefCell::new(None);
}

/// State of the local clock.
#[derive(Default, Clone)]
pub struct LocalClock {
    /// true if the clock is frozen on this thread, otherwise false.
    pub frozen: bool,
    /// The current local time.
    pub time: Timepoint,
    /// The shared clock.
    pub shared_clock: Arc<SharedClock>,
}
