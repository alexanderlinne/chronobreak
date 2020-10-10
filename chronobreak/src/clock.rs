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

macro_rules! match_clock_strategy {
    (Sys => $sys:expr, Manual => $man:expr, AutoInc => $aut:expr,) => {
        match crate::clock::strategy() {
            crate::clock::ClockStrategy::Sys => $sys,
            crate::clock::ClockStrategy::Manual => $man,
            crate::clock::ClockStrategy::AutoInc => $aut,
        }
    };
}

// Returns the current clock strategy.
pub fn strategy() -> ClockStrategy {
    CLOCK_MODE.with(|v| v.borrow().to_owned().unwrap_or(ClockStrategy::Sys))
}

// Sets the clock to manual strategy for the current thread. This function
// must not be called again before the returned guard is dropped. Dropping the
// guard resets the strategy to the system clock and the internal values of the
// manual and auto_inc clock to Duration::default().
pub fn manual() -> Result<ClockGuard, ()> {
    set_strategy(ClockStrategy::Manual)
}

// Sets the clock to auto_inc strategy for the current thread. This function
// must not be called again before the returned guard is dropped. Dropping the
// guard resets the strategy to the system clock and the internal values of the
// manual and auto_inc clock to Duration::default().
pub fn auto_inc() -> Result<ClockGuard, ()> {
    set_strategy(ClockStrategy::AutoInc)
}

fn set_strategy(mode: ClockStrategy) -> Result<ClockGuard, ()> {
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

pub(crate) fn raw() -> Option<ClockStrategy> {
    CLOCK_MODE.with(|v| v.borrow().to_owned())
}

pub(crate) fn from_raw(raw: Option<ClockStrategy>) {
    CLOCK_MODE.with(|v| *v.borrow_mut() = raw)
}

pub fn set(dur: time::Duration) {
    match_clock_strategy! {
        Sys => panic!{"chronobreak::clock::set requires the clock to be mocked"},
        Manual => MANUAL.with(|v| *v.borrow().lock().unwrap() = dur),
        AutoInc => AUTO_INC.with(|v| *v.borrow_mut() = dur),
    }
}

pub fn reset() {
    set(time::Duration::default())
}

pub fn get() -> time::Duration {
    match_clock_strategy! {
        Sys => panic!{"chronobreak::clock::get requires the clock to be mocked"},
        Manual => MANUAL.with(|v| *v.borrow().lock().unwrap()),
        AutoInc => AUTO_INC.with(|v| *v.borrow()),
    }
}

pub fn fetch_add(dur: time::Duration) -> time::Duration {
    match_clock_strategy! {
        Sys => panic!{"chronobreak::clock::fetch_add requires the clock to be mocked"},
        Manual => MANUAL.with(|v| {
            let v = v.borrow_mut();
            let mut v = v.lock().unwrap();
            let result = *v;
            *v += dur;
            result
        }),
        AutoInc => AUTO_INC.with(|v| {
            let mut v = v.borrow_mut();
            let result = *v;
            *v += dur;
            result
        }),
    }
}

#[macro_export]
macro_rules! assert_clock_eq {
    ($dur:expr) => {
        match ::chronobreak::clock::strategy() {
            ::chronobreak::clock::ClockStrategy::Sys => {
                panic! {"assert_clock_eq! {...} needs the clock to be mocked!"}
            }
            ::chronobreak::clock::ClockStrategy::Manual => {
                assert_eq!(::chronobreak::clock::get(), $dur)
            }
            ::chronobreak::clock::ClockStrategy::AutoInc => {
                assert_eq!(::chronobreak::clock::get(), $dur)
            }
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

pub(crate) mod manual {
    use super::*;
    use crate::mock::std::time;

    #[allow(dead_code)]
    pub(crate) type Raw = Arc<Mutex<time::Duration>>;

    #[allow(dead_code)]
    pub(crate) fn raw() -> Raw {
        MANUAL.with(|v| v.borrow().clone())
    }

    #[allow(dead_code)]
    pub(crate) fn from_raw(raw: Raw) {
        MANUAL.with(|v| *v.borrow_mut() = raw)
    }
}

pub(crate) mod auto_inc {
    use super::*;
    use crate::mock::std::time;

    #[allow(dead_code)]
    pub(crate) type Raw = time::Duration;

    #[allow(dead_code)]
    pub(crate) fn raw() -> Raw {
        AUTO_INC.with(|v| *v.borrow())
    }

    #[allow(dead_code)]
    pub(crate) fn from_raw(raw: Raw) {
        AUTO_INC.with(|v| *v.borrow_mut() = raw)
    }
}
