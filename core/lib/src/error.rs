use thiserror::Error;

/// Error type that may be returned by chronobreak's APIs.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ChronobreakError {
    /// [`clock::mock`](../clock/fn.mock.html) or [`clock::frozen`](../clock/fn.frozen.html) was called while the clock was already
    /// mocked.
    #[error("the clock is already mocked")]
    AlreadyInitialized,
}
