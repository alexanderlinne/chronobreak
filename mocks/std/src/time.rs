use chronobreak::{clock, mock};
use std::{cmp, fmt, hash, ops, time};

pub use time::{Duration, SystemTimeError, UNIX_EPOCH};

/// **Mock** of [`std::time::Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html)
#[derive(Copy, Clone)]
pub struct Instant(mock::Mock<time::Instant, clock::Timepoint>);

impl Instant {
    mock::constants![];

    pub fn now() -> Self {
        Self(mock::Mock::new(time::Instant::now, clock::get))
    }

    pub fn duration_since(&self, earlier: Self) -> Duration {
        mock::apply!((self, earlier), |(now, earlier)| now
            .duration_since(earlier))
    }

    pub fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
        mock::apply!((self, earlier), |(now, earlier)| now
            .checked_duration_since(earlier))
    }

    pub fn saturating_duration_since(&self, earlier: Self) -> Duration {
        mock::apply!((self, earlier), |(now, earlier)| now
            .saturating_duration_since(earlier))
    }

    pub fn elapsed(&self) -> Duration {
        mock::apply!(self, |actual| actual.elapsed(), |_| Self::now() - *self)
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        mock::map!(self, |v| v.checked_add(duration))
            .flatten()
            .map(&Self)
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        mock::map!(self, |v| v.checked_sub(duration))
            .flatten()
            .map(&Self)
    }
}

impl Ord for Instant {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        mock::apply!((self, &rhs), |(lhs, rhs)| lhs.cmp(rhs))
    }
}

impl PartialOrd<Instant> for Instant {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        mock::apply!((self, &rhs), |(lhs, rhs)| lhs.partial_cmp(rhs))
    }
}

impl Eq for Instant {}

impl PartialEq<Instant> for Instant {
    fn eq(&self, rhs: &Self) -> bool {
        mock::apply!((self, &rhs), |(lhs, rhs)| lhs.eq(rhs))
    }
}

impl hash::Hash for Instant {
    fn hash<H>(&self, h: &mut H)
    where
        H: hash::Hasher,
    {
        mock::apply!(self, |v| v.hash(h))
    }
}

impl ops::Add<Duration> for Instant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        Self(mock::map!(self, |v| v.add(rhs)))
    }
}

impl ops::AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        mock::apply!(self, |mut v| v.add_assign(rhs))
    }
}

impl ops::Sub<Duration> for Instant {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self {
        Self(mock::map!(self, |v| v.sub(rhs)))
    }
}

impl ops::SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        mock::apply!(self, |mut v| v.sub_assign(rhs))
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
        mock::apply!(self, |v| v.fmt(f))
    }
}

/// **Mock** of [`std::time::SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html)
#[derive(Copy, Clone)]
pub struct SystemTime(mock::Mock<time::SystemTime, clock::Timepoint>);

impl SystemTime {
    mock::constants![(
        UNIX_EPOCH,
        time::SystemTime::UNIX_EPOCH,
        clock::Timepoint::START
    )];

    pub fn now() -> Self {
        Self(mock::Mock::new(time::SystemTime::now, clock::get))
    }

    pub fn duration_since(&self, earlier: Self) -> Result<Duration, SystemTimeError> {
        mock::apply!(
            (self, earlier),
            |(now, earlier)| now.duration_since(earlier),
            |(now, earlier)| Ok(now.duration_since(earlier))
        )
    }

    pub fn elapsed(&self) -> Result<Duration, SystemTimeError> {
        mock::apply!(self, |v| v.elapsed(), |_| Self::now().duration_since(*self))
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        mock::map!(self, |v| v.checked_add(duration))
            .flatten()
            .map(&Self)
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        mock::map!(self, |v| v.checked_sub(duration))
            .flatten()
            .map(&Self)
    }
}

impl Ord for SystemTime {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        mock::apply!((self, &rhs), |(lhs, rhs)| lhs.cmp(rhs))
    }
}

impl PartialOrd<SystemTime> for SystemTime {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        mock::apply!((self, &rhs), |(lhs, rhs)| lhs.partial_cmp(rhs))
    }
}

impl Eq for SystemTime {}

impl PartialEq<SystemTime> for SystemTime {
    fn eq(&self, rhs: &Self) -> bool {
        mock::apply!((self, &rhs), |(lhs, rhs)| lhs.eq(rhs))
    }
}

impl hash::Hash for SystemTime {
    fn hash<H>(&self, h: &mut H)
    where
        H: hash::Hasher,
    {
        mock::apply!(self, |v| v.hash(h))
    }
}

impl ops::Add<Duration> for SystemTime {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        Self(mock::map!(self, |v| v.add(rhs)))
    }
}

impl ops::AddAssign<Duration> for SystemTime {
    fn add_assign(&mut self, rhs: Duration) {
        mock::apply!(self, |mut v| v.add_assign(rhs))
    }
}

impl ops::Sub<Duration> for SystemTime {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self {
        Self(mock::map!(self, |v| v.sub(rhs)))
    }
}

impl ops::SubAssign<Duration> for SystemTime {
    fn sub_assign(&mut self, rhs: Duration) {
        mock::apply!(self, |mut v| v.sub_assign(rhs))
    }
}

impl fmt::Debug for SystemTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        mock::apply!(self, |v| v.fmt(f))
    }
}
