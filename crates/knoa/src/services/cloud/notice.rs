//! What sync has to tell the user that isn't an error of the sync itself.

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

/// Why the server refused a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    /// It broke the server's rules, like a value past its column's limit.
    Invalid,
    /// It was too large for the server even alone.
    TooLarge,
}
