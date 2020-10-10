use std::cmp;
use std::time;

pub use time::Duration;

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

#[derive(Copy, Clone, Debug)]
pub enum Instant {
    Actual(time::Instant),
    Mocked(Duration),
}

impl Instant {
    pub fn now() -> Self {
        match_clock_strategy! {
            Sys => Self::Actual(time::Instant::now()),
            Manual => Self::Mocked(crate::clock::get()),
            AutoInc => Self::Mocked(crate::clock::get()),
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
