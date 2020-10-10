use crate::clock::ClockStrategy;
use std::cmp;
use std::time;

pub use time::Duration;

#[derive(Copy, Clone, Debug)]
pub enum Instant {
    Sys(time::Instant),
    Mocked(Duration),
}

impl Instant {
    pub fn now() -> Self {
        match ClockStrategy::current() {
            ClockStrategy::Sys => Self::Sys(time::Instant::now()),
            ClockStrategy::Manual => Self::Mocked(crate::clock::manual::get()),
            ClockStrategy::AutoInc => Self::Mocked(crate::clock::auto_inc::get()),
        }
    }

    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        match self {
            Self::Sys(now) => match earlier {
                Self::Sys(earlier) => now.saturating_duration_since(earlier),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
            Self::Mocked(now) => match earlier {
                Self::Mocked(earlier) => now.checked_sub(earlier).unwrap_or_default(),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
        }
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        match self {
            Self::Sys(instant) => instant.checked_add(duration).map(&Self::Sys),
            Self::Mocked(current_time) => current_time.checked_add(duration).map(&Self::Mocked),
        }
    }
}

impl Ord for Instant {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        println!("time::Instant::cmp");
        match self {
            Self::Sys(lhs) => match rhs {
                Self::Sys(rhs) => lhs.cmp(rhs),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
            Self::Mocked(lhs) => match rhs {
                Self::Mocked(rhs) => lhs.cmp(rhs),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
        }
    }
}

impl PartialOrd<Instant> for Instant {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        match self {
            Self::Sys(lhs) => match rhs {
                Self::Sys(rhs) => lhs.partial_cmp(rhs),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
            Self::Mocked(lhs) => match rhs {
                Self::Mocked(rhs) => lhs.partial_cmp(rhs),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
        }
    }
}

impl Eq for Instant {}

impl PartialEq<Instant> for Instant {
    fn eq(&self, rhs: &Self) -> bool {
        match self {
            Self::Sys(lhs) => match rhs {
                Self::Sys(rhs) => lhs.eq(rhs),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
            Self::Mocked(lhs) => match rhs {
                Self::Mocked(rhs) => lhs.eq(rhs),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
        }
    }
}
