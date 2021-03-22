use crate::clock;

pub use chronobreak_derive::{apply, constants, map};

pub enum Mock<Actual, Mocked> {
    Actual(Actual),
    Mocked(Mocked),
    Constant(usize),
}

impl<Actual, Mocked> Mock<Actual, Mocked> {
    pub fn new<ActualFn, MockedFn>(actual_fn: ActualFn, mocked_fn: MockedFn) -> Self
    where
        ActualFn: FnOnce() -> Actual,
        MockedFn: FnOnce() -> Mocked,
    {
        if clock::is_mocked() {
            Mock::Mocked(mocked_fn())
        } else {
            Mock::Actual(actual_fn())
        }
    }

    pub const fn actual(value: Actual) -> Self {
        Self::Actual(value)
    }

    pub const fn mocked(value: Mocked) -> Self {
        Self::Mocked(value)
    }

    pub const fn constant(id: usize) -> Self {
        Self::Constant(id)
    }
}

impl<Actual, Mocked> Mock<Option<Actual>, Option<Mocked>> {
    pub fn flatten(self) -> Option<Mock<Actual, Mocked>> {
        match self {
            Self::Actual(Some(actual)) => Some(Mock::Actual(actual)),
            Self::Mocked(Some(mocked)) => Some(Mock::Mocked(mocked)),
            _ => None,
        }
    }
}

impl<Actual, Mocked> Copy for Mock<Actual, Mocked>
where
    Actual: Copy,
    Mocked: Copy,
{
}

impl<Actual, Mocked> Clone for Mock<Actual, Mocked>
where
    Actual: Clone,
    Mocked: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Actual(actual) => Self::Actual(actual.clone()),
            Self::Mocked(mocked) => Self::Mocked(mocked.clone()),
            Self::Constant(id) => Self::Constant(*id),
        }
    }
}
