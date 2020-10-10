use crate::mock::std::time;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

thread_local! {
    // The current clock strategy.
    static CLOCK_MODE: RefCell<Option<ClockStrategy>> = RefCell::new(None);

    // The current time in nanoseconds of the manual clock.
    static MANUAL: RefCell<Arc<Mutex<time::Duration>>> = RefCell::new(Arc::new(Mutex::new(time::Duration::default())));

    // The carrent time in nanoseconds of the auto incrementing clock.
    static AUTO_INC: RefCell<time::Duration> = RefCell::new(time::Duration::default());
}

// Specifies the underlying implementation used by the mocked clock. By default
// the system clock is used.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ClockStrategy {
    // The test mock uses the system time. (default)
    Sys,
    // The mocked Instant uses a global nanosecond counter that does not
    // increase unless the user manually requests it.
    Manual,
    // The mocked Instant uses a thread-local counters that automatically
    // increment when e.g. the thread calls thread::sleep. When one thread
    // joins another, the clock of the caller is set to the maximum of both
    // clocks.
    AutoInc,
}

impl ClockStrategy {
    // Returns the current clock mode.
    pub fn current() -> Self {
        CLOCK_MODE.with(|v| v.borrow().to_owned().unwrap_or(Self::Sys))
    }

    // Sets the clock mode globally. This may only be called once per test.
    pub fn set(mode: ClockStrategy) -> Result<ClockGuard, ()> {
        CLOCK_MODE.with(|v| {
            let mut clock_mode = v.borrow_mut();
            if (*clock_mode).is_some() {
                Err(())
            } else {
                *clock_mode = Some(mode);
                Ok(ClockGuard {})
            }
        })
    }

    // Returns the current clock mode or None if it ClockStrategy::set has not
    // been called yet.
    #[allow(dead_code)]
    pub(crate) fn raw() -> Option<Self> {
        CLOCK_MODE.with(|v| v.borrow().to_owned())
    }

    // Returns the current clock mode or None if it ClockStrategy::set has not
    // been called yet.
    #[allow(dead_code)]
    pub(crate) fn from_raw(raw: Option<Self>) {
        CLOCK_MODE.with(|v| *v.borrow_mut() = raw)
    }
}

#[macro_export]
macro_rules! assert_clock_eq {
    ($dur:expr) => {
        match ClockStrategy::current() {
            ClockStrategy::Sys => panic! {"assert_clock_eq! {...} needs the clock to be mocked!"},
            ClockStrategy::Manual => assert_eq!(manual::get(), $dur),
            ClockStrategy::AutoInc => assert_eq!(auto_inc::get(), $dur),
        };
    };
}

#[must_use]
pub struct ClockGuard {}

impl Drop for ClockGuard {
    fn drop(&mut self) {
        CLOCK_MODE.with(|v| *v.borrow_mut() = None);
        MANUAL.with(|v| *v.borrow().lock().unwrap() = time::Duration::default());
        AUTO_INC.with(|v| *v.borrow_mut() = time::Duration::default());
    }
}

pub mod manual {
    use super::*;
    use crate::mock::std::time;

    pub fn set(dur: time::Duration) {
        assert_eq!(ClockStrategy::current(), ClockStrategy::Manual);
        MANUAL.with(|v| *v.borrow().lock().unwrap() = dur)
    }

    pub fn get() -> time::Duration {
        assert_eq!(ClockStrategy::current(), ClockStrategy::Manual);
        MANUAL.with(|v| *v.borrow().lock().unwrap())
    }

    pub fn fetch_add(dur: time::Duration) -> time::Duration {
        assert_eq!(ClockStrategy::current(), ClockStrategy::Manual);
        MANUAL.with(|v| {
            let v = v.borrow_mut();
            let mut v = v.lock().unwrap();
            let result = *v;
            *v += dur;
            result
        })
    }

    #[allow(dead_code)]
    pub(crate) fn raw() -> Arc<Mutex<time::Duration>> {
        MANUAL.with(|v| v.borrow().clone())
    }

    #[allow(dead_code)]
    pub(crate) fn from_raw(raw: Arc<Mutex<time::Duration>>) {
        MANUAL.with(|v| *v.borrow_mut() = raw)
    }
}

pub mod auto_inc {
    use super::*;
    use crate::mock::std::time;

    #[allow(dead_code)]
    pub(crate) type Raw = time::Duration;

    pub fn get() -> time::Duration {
        assert_eq!(ClockStrategy::current(), ClockStrategy::AutoInc);
        AUTO_INC.with(|v| *v.borrow())
    }

    pub fn fetch_add(dur: time::Duration) -> time::Duration {
        assert_eq!(ClockStrategy::current(), ClockStrategy::AutoInc);
        AUTO_INC.with(|v| {
            let mut v = v.borrow_mut();
            let result = *v;
            *v += dur;
            result
        })
    }

    #[allow(dead_code)]
    pub(crate) fn raw() -> Raw {
        AUTO_INC.with(|v| *v.borrow())
    }

    #[allow(dead_code)]
    pub(crate) fn from_raw(raw: Raw) {
        AUTO_INC.with(|v| *v.borrow_mut() = raw)
    }
}
