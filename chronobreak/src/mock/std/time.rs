use crate::clock;
use std::{cmp, fmt, hash, ops, time};

pub use time::{Duration, SystemTimeError, UNIX_EPOCH};

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
#[derive(Copy, Clone)]
pub enum Instant {
    Actual(time::Instant),
    Mocked(Duration),
}

impl Instant {
    pub fn now() -> Self {
        if clock::is_mocked() {
            Self::Mocked(clock::get())
        } else {
            Self::Actual(time::Instant::now())
        }
    }

    pub fn duration_since(&self, earlier: Self) -> Duration {
        instant_delegate! {self, now, earlier, now.duration_since(earlier), now.checked_sub(earlier).expect("supplied instant is later than self")}
    }

    pub fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
        instant_delegate! {self, now, earlier, now.checked_duration_since(earlier), now.checked_sub(earlier)}
    }

    pub fn saturating_duration_since(&self, earlier: Self) -> Duration {
        instant_delegate! {self, now, earlier, now.saturating_duration_since(earlier), now.checked_sub(earlier).unwrap_or_default()}
    }

    pub fn elapsed(&self) -> Duration {
        match self {
            Self::Actual(actual) => actual.elapsed(),
            Self::Mocked(_) => Self::now() - *self,
        }
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        match self {
            Self::Actual(actual) => actual.checked_add(duration).map(&Self::Actual),
            Self::Mocked(dur) => dur.checked_add(duration).map(&Self::Mocked),
        }
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        match self {
            Self::Actual(actual) => actual.checked_add(duration).map(&Self::Actual),
            Self::Mocked(dur) => dur.checked_sub(duration).map(&Self::Mocked),
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

impl hash::Hash for Instant {
    fn hash<H>(&self, h: &mut H)
    where
        H: hash::Hasher,
    {
        match self {
            Self::Actual(instant) => instant.hash(h),
            Self::Mocked(dur) => dur.hash(h),
        }
    }
}

impl ops::Add<Duration> for Instant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        match self {
            Self::Actual(actual) => Self::Actual(actual.add(rhs)),
            Self::Mocked(dur) => Self::Mocked(dur.add(rhs)),
        }
    }
}

impl ops::AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        match self {
            Self::Actual(actual) => actual.add_assign(rhs),
            Self::Mocked(dur) => dur.add_assign(rhs),
        }
    }
}

impl ops::Sub<Duration> for Instant {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self {
        match self {
            Self::Actual(actual) => Self::Actual(actual.sub(rhs)),
            Self::Mocked(dur) => Self::Mocked(dur.sub(rhs)),
        }
    }
}

impl ops::SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        match self {
            Self::Actual(actual) => actual.sub_assign(rhs),
            Self::Mocked(dur) => dur.sub_assign(rhs),
        }
    }
}

impl ops::Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Duration {
        self.duration_since(rhs)
    }
}

impl fmt::Debug for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Actual(actual) => actual.fmt(f),
            Self::Mocked(dur) => dur.fmt(f),
        }
    }
}

macro_rules! system_time_delegate {
    ($self:ident, $lhs:ident, $rhs:ident, $actual:expr, $mocked:expr) => {
        let self_value = $self.handle_unix_epoch();
        let rhs_value = *$rhs.handle_unix_epoch();
        match self_value {
            Self::Actual($lhs) => match rhs_value {
                Self::Actual($rhs) => $actual,
                Self::UnixEpoch => panic!("Found UnixEpoch unexpectedly!"),
                _ => panic!("Found incompatible Instant unexpectedly!"),
            },
            Self::Mocked($lhs) => match rhs_value {
                Self::Mocked($rhs) => $mocked,
                Self::UnixEpoch => panic!("Found UnixEpoch unexpectedly!"),
                _ => panic!("Found incompatible Instant unexpectedly!"),
            },
            Self::UnixEpoch => panic!("Found UnixEpoch unexpectedly!"),
        }
    };
    ($self:ident, $lhs:ident, $rhs:ident, $e:expr) => {
        system_time_delegate! {$self, $lhs, $rhs, $e, $e}
    };
}

/// **Mock** of [`std::time::SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html)
#[derive(Copy, Clone)]
pub enum SystemTime {
    Actual(time::SystemTime),
    Mocked(Duration),
    UnixEpoch,
}

impl SystemTime {
    pub const UNIX_EPOCH: Self = Self::UnixEpoch;
    const ACTUAL_UNIX_EPOCH: Self = Self::Actual(time::SystemTime::UNIX_EPOCH);
    const MOCKED_UNIX_EPOCH: Self = Self::Mocked(Duration::from_secs(0));

    fn handle_unix_epoch(&self) -> &Self {
        if let Self::UnixEpoch = *self {
            if clock::is_mocked() {
                &Self::MOCKED_UNIX_EPOCH
            } else {
                &Self::ACTUAL_UNIX_EPOCH
            }
        } else {
            self
        }
    }

    fn handle_unix_epoch_mut(&mut self) {
        *self = *self.handle_unix_epoch();
    }

    pub fn now() -> Self {
        if clock::is_mocked() {
            Self::Mocked(clock::get())
        } else {
            Self::Actual(time::SystemTime::now())
        }
    }

    pub fn duration_since(&self, earlier: Self) -> Result<Duration, SystemTimeError> {
        system_time_delegate! {self, now, earlier, now.duration_since(earlier), Ok(now.checked_sub(earlier).expect("supplied instant is later than self"))}
    }

    pub fn elapsed(&self) -> Result<Duration, SystemTimeError> {
        let value = self.handle_unix_epoch();
        match value {
            Self::Actual(actual) => actual.elapsed(),
            Self::Mocked(_) => Self::now().duration_since(*self),
            Self::UnixEpoch => panic!("Found incompatible Instant unexpectedly!"),
        }
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        let value = self.handle_unix_epoch();
        match value {
            Self::Actual(actual) => actual.checked_add(duration).map(&Self::Actual),
            Self::Mocked(dur) => dur.checked_add(duration).map(&Self::Mocked),
            Self::UnixEpoch => panic!("Found incompatible Instant unexpectedly!"),
        }
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        let value = self.handle_unix_epoch();
        match value {
            Self::Actual(actual) => actual.checked_sub(duration).map(&Self::Actual),
            Self::Mocked(dur) => dur.checked_sub(duration).map(&Self::Mocked),
            Self::UnixEpoch => panic!("Found incompatible Instant unexpectedly!"),
        }
    }
}

impl Ord for SystemTime {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        system_time_delegate! {self, lhs, rhs, lhs.cmp(&rhs)}
    }
}

impl PartialOrd<SystemTime> for SystemTime {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        system_time_delegate! {self, lhs, rhs, lhs.partial_cmp(&rhs)}
    }
}

impl Eq for SystemTime {}

impl PartialEq<SystemTime> for SystemTime {
    fn eq(&self, rhs: &Self) -> bool {
        system_time_delegate! {self, lhs, rhs, lhs.eq(&rhs)}
    }
}

impl hash::Hash for SystemTime {
    fn hash<H>(&self, h: &mut H)
    where
        H: hash::Hasher,
    {
        let value = self.handle_unix_epoch();
        match value {
            Self::Actual(instant) => instant.hash(h),
            Self::Mocked(dur) => dur.hash(h),
            Self::UnixEpoch => panic!("Found incompatible Instant unexpectedly!"),
        }
    }
}

impl ops::Add<Duration> for SystemTime {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        let value = self.handle_unix_epoch();
        match value {
            Self::Actual(actual) => Self::Actual(actual.add(rhs)),
            Self::Mocked(dur) => Self::Mocked(dur.add(rhs)),
            Self::UnixEpoch => panic!("Found incompatible Instant unexpectedly!"),
        }
    }
}

impl ops::AddAssign<Duration> for SystemTime {
    fn add_assign(&mut self, rhs: Duration) {
        self.handle_unix_epoch_mut();
        match self {
            Self::Actual(actual) => actual.add_assign(rhs),
            Self::Mocked(dur) => dur.add_assign(rhs),
            Self::UnixEpoch => panic!("Found incompatible Instant unexpectedly!"),
        }
    }
}

impl ops::Sub<Duration> for SystemTime {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self {
        let value = self.handle_unix_epoch();
        match value {
            Self::Actual(actual) => Self::Actual(actual.sub(rhs)),
            Self::Mocked(dur) => Self::Mocked(dur.sub(rhs)),
            Self::UnixEpoch => panic!("Found incompatible Instant unexpectedly!"),
        }
    }
}

impl ops::SubAssign<Duration> for SystemTime {
    fn sub_assign(&mut self, rhs: Duration) {
        self.handle_unix_epoch_mut();
        match self {
            Self::Actual(actual) => actual.sub_assign(rhs),
            Self::Mocked(dur) => dur.sub_assign(rhs),
            Self::UnixEpoch => panic!("Found incompatible Instant unexpectedly!"),
        }
    }
}

impl fmt::Debug for SystemTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let value = self.handle_unix_epoch();
        match value {
            Self::Actual(actual) => actual.fmt(f),
            Self::Mocked(dur) => dur.fmt(f),
            Self::UnixEpoch => panic!("Found incompatible Instant unexpectedly!"),
        }
    }
}
