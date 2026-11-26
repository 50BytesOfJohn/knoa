//! What Cloud mode has to tell the user, as data the UI puts in words when it shows it,
//! so it follows the app's language.

use knoa_core::protocol::{Account, Organizer, Plan};

/// Something the plan or the server holds back. Each kind needs a different fix, so the
/// sidebar's status card tells them apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notice {
    /// New bookmarks past the plan's limit stayed on this Mac.
    PlanLimit { limit: u64, unsent: usize },
    /// The server refused some changes, which stayed on this Mac.
    Rejected(Rejection),
    /// This month's organizing allowance is used up; the server keeps the requests.
    AllowanceUsedUp,
    /// A free account used this month's cloud organizing. Nothing waits for next month,
    /// so the way on is Pro or a key of the user's own.
    FreeAllowanceUsedUp { allowance: u64 },
}

impl Notice {
    /// Read from the account rather than remembered, so it goes away by itself with the
    /// next month or an upgrade.
    pub fn of_allowance(account: &Account) -> Option<Notice> {
        let allowance = account.limits.organized_per_month;
        match account.plan {
            // The server moves a free account to this Mac once its allowance is gone, also
            // through runs still pending or spend, which the count of organized ones
            // doesn't show. An allowance of 0 is a server from before free accounts had one.
            Plan::Free if allowance > 0 && account.organizer == Organizer::Device => {
                Some(Notice::FreeAllowanceUsedUp { allowance })
            }
            Plan::Pro
                if account.organizer == Organizer::Cloud
                    && account.usage.organized_this_month >= allowance =>
            {
                Some(Notice::AllowanceUsedUp)
            }
            Plan::Free | Plan::Pro => None,
        }
    }
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

#[cfg(test)]
mod tests {
    use knoa_core::protocol::{AccountUsage, Limits};

    use super::*;

    fn account(plan: Plan, organizer: Organizer, organized: u64, allowance: u64) -> Account {
        Account {
            user_id: "user".into(),
            email: None,
            plan,
            organizer,
            limits: Limits {
                bookmarks: 10_000,
                organized_per_month: allowance,
            },
            usage: AccountUsage {
                bookmarks: 0,
                organized_this_month: organized,
            },
            billing: None,
            language: None,
        }
    }

    #[test]
    fn free_account_that_used_its_allowance_is_told_how_to_go_on() {
        let account = account(Plan::Free, Organizer::Device, 25, 25);
        assert_eq!(
            Notice::of_allowance(&account),
            Some(Notice::FreeAllowanceUsedUp { allowance: 25 })
        );
    }

    #[test]
    fn free_account_without_a_cloud_allowance_hears_nothing() {
        let account = account(Plan::Free, Organizer::Device, 0, 0);
        assert_eq!(Notice::of_allowance(&account), None);
    }

    #[test]
    fn free_account_moved_to_this_mac_before_its_count_shows_it_is_told_too() {
        let account = account(Plan::Free, Organizer::Device, 3, 25);
        assert_eq!(
            Notice::of_allowance(&account),
            Some(Notice::FreeAllowanceUsedUp { allowance: 25 })
        );
    }

    #[test]
    fn free_account_the_cloud_still_organizes_hears_nothing() {
        let account = account(Plan::Free, Organizer::Cloud, 25, 25);
        assert_eq!(Notice::of_allowance(&account), None);
    }

    #[test]
    fn pro_account_that_used_its_allowance_waits_for_more() {
        let account = account(Plan::Pro, Organizer::Cloud, 3_000, 3_000);
        assert_eq!(
            Notice::of_allowance(&account),
            Some(Notice::AllowanceUsedUp)
        );
    }
}
