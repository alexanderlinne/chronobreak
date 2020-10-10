use std::cmp;
use std::time;

pub use time::Duration;

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
        match self {
            Self::Actual(now) => match earlier {
                Self::Actual(earlier) => now.saturating_duration_since(earlier),
                _ => panic!("Found incompatible Instant unexpectedly!"),
            },
            Self::Mocked(now) => match earlier {
                Self::Mocked(earlier) => now.checked_sub(earlier).unwrap_or_default(),
                _ => panic!("Found incompatible Instant unexpectedly!"),
            },
        }
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
        println!("time::Instant::cmp");
        match self {
            Self::Actual(lhs) => match rhs {
                Self::Actual(rhs) => lhs.cmp(rhs),
                _ => panic!("Found incompatible Instant unexpectedly!"),
            },
            Self::Mocked(lhs) => match rhs {
                Self::Mocked(rhs) => lhs.cmp(rhs),
                _ => panic!("Found incompatible Instant unexpectedly!"),
            },
        }
    }
}

impl PartialOrd<Instant> for Instant {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        match self {
            Self::Actual(lhs) => match rhs {
                Self::Actual(rhs) => lhs.partial_cmp(rhs),
                _ => panic!("Found incompatible Instant unexpectedly!"),
            },
            Self::Mocked(lhs) => match rhs {
                Self::Mocked(rhs) => lhs.partial_cmp(rhs),
                _ => panic!("Found incompatible Instant unexpectedly!"),
            },
        }
    }
}

impl Eq for Instant {}

impl PartialEq<Instant> for Instant {
    fn eq(&self, rhs: &Self) -> bool {
        match self {
            Self::Actual(lhs) => match rhs {
                Self::Actual(rhs) => lhs.eq(rhs),
                _ => panic!("Found incompatible Instant unexpectedly!"),
            },
            Self::Mocked(lhs) => match rhs {
                Self::Mocked(rhs) => lhs.eq(rhs),
                _ => panic!("Found incompatible Instant unexpectedly!"),
            },
        }
    }
}
