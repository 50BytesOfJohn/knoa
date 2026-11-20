//! What Cloud mode has to tell the user, as data the UI puts in words when it shows it,
//! so it follows the app's language.

/// Something the plan or the server holds back. Each kind needs a different fix, so the
/// sidebar's status card tells them apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notice {
    /// New bookmarks past the plan's limit stayed on this Mac.
    PlanLimit { limit: u64, unsent: usize },
    /// The server refused some changes, which stayed on this Mac.
    Rejected(Rejection),
    /// This month's organizing allowance is used up.
    AllowanceUsedUp,
}

/// Why signing in, or staying signed in, didn't work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    SignInFailed(String),
    /// The cloud refused the sign-in, which signed this Mac out.
    SignedOut,
    /// Unlinking the library from the other account failed.
    UnlinkFailed(String),
}

/// Why sync stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    /// The server answered none of the changes sent.
    Unanswered,
    Failed(String),
}

/// Why the server refused a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    /// It broke the server's rules, like a value past its column's limit.
    Invalid,
    /// It was too large for the server even alone.
    TooLarge,
}
