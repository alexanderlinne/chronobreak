use crate::clock;
use std::cmp;
use std::ops::Add;
use std::time;

pub use time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

macro_rules! instant_delegate {
    ($self:ident, $lhs:ident, $rhs:ident, $actual:expr, $mocked:expr) => {
        match $self {
            Self::Actual($lhs) => match $rhs {
                Self::Actual($rhs) => $actual,
                _ => panic!("Found incompatible Instant unexpectedly!"),
            },
            Self::Mocked($lhs) => match $rhs {
                Self::Mocked($rhs) => $mocked,
                _ => panic!("Found incompatible Instant unexpectedly!"),
            },
        }
    };
    ($self:ident, $lhs:ident, $rhs:ident, $e:expr) => {
        instant_delegate! {$self, $lhs, $rhs, $e, $e}
    };
}

/// **Mock** of [`std::time::Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html)
#[derive(Copy, Clone, Debug)]
pub enum Instant {
    Actual(time::Instant),
    Mocked(Duration),
}

impl Into<time::Instant> for Instant {
    fn into(self) -> time::Instant {
        match self {
            Self::Actual(instant) => instant,
            Self::Mocked(_) => {
                panic! {"chronobreak: Cannot convert mocked Instant into std::time::Instant"}
            }
        }
    }
}

impl From<Instant> for Duration {
    fn from(current_time: Instant) -> Self {
        match current_time {
            Instant::Actual(_) => {
                panic! {"chronobreak: Cannot convert non-mocked Instant into Duration"}
            }
            Instant::Mocked(current_time) => current_time,
        }
    }
}

impl From<Duration> for Instant {
    fn from(current_time: Duration) -> Self {
        if clock::is_mocked() {
            Instant::Mocked(current_time)
        } else {
            panic! {"chronobreak: Cannot convert Duration into non-mocked Instant"}
        }
    }
}

impl Instant {
    pub fn now() -> Self {
        if clock::is_mocked() {
            Self::Mocked(clock::get())
        } else {
            Self::Actual(time::Instant::now())
        }
    }

    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        instant_delegate! {self, now, earlier, now.saturating_duration_since(earlier), now.checked_sub(earlier).unwrap_or_default()}
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        match self {
            Self::Actual(instant) => instant.checked_add(duration).map(&Self::Actual),
            Self::Mocked(current_time) => current_time.checked_add(duration).map(&Self::Mocked),
        }
    }
}

impl Ord for Instant {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        instant_delegate! {self, lhs, rhs, lhs.cmp(rhs)}
    }
}

impl PartialOrd<Instant> for Instant {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        instant_delegate! {self, lhs, rhs, lhs.partial_cmp(rhs)}
    }
}

impl Eq for Instant {}

impl PartialEq<Instant> for Instant {
    fn eq(&self, rhs: &Self) -> bool {
        instant_delegate! {self, lhs, rhs, lhs.eq(rhs)}
    }
}

impl Add<Duration> for Instant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        match self {
            Self::Actual(instant) => Self::Actual(instant + rhs),
            Self::Mocked(current_time) => Self::Mocked(current_time + rhs),
        }
    }
}
