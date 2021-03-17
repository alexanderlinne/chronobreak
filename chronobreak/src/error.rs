use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ChronobreakError {
    #[error("the clock is already mocked")]
    AlreadyInitialized,
}
