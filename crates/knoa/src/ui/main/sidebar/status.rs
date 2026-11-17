//! What the sidebar's status card says: the most urgent thing the AI or sync has to tell,
//! or nothing while all is well.

use crate::{
    services::{cloud::SyncStatus, organizer::AiStatus},
    state::batch::Progress,
};

/// One message for the card, most urgent first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Status {
    AiNeedsAttention,
    SyncStopped,
    /// Signed in, but the library is linked to another account, so nothing syncs.
    SyncPaused,
    PlanLimit,
    Organizing(Progress),
    AiOff,
    Offline,
}

/// Where Cloud mode stands, as far as the card cares.
#[derive(Debug, Clone, Copy)]
pub(super) enum CloudState<'a> {
    /// Signed out, or a build without a cloud: sync has nothing to say.
    Local,
    OtherAccount,
    SignedIn {
        sync: &'a SyncStatus,
        /// The plan's limit holds something back.
        plan_limit: bool,
    },
}

impl Status {
    /// The message to show, if any. Sync speaks only while signed in.
    pub fn of(ai: &AiStatus, cloud: CloudState<'_>, batch: Option<Progress>) -> Option<Status> {
        let (sync, plan_limit) = match cloud {
            CloudState::SignedIn { sync, plan_limit } => (Some(sync), plan_limit),
            CloudState::Local | CloudState::OtherAccount => (None, false),
        };
        let ai_works = matches!(ai, AiStatus::Ready | AiStatus::Cloud);
        if matches!(ai, AiStatus::NeedsAttention(_)) {
            Some(Status::AiNeedsAttention)
        } else if matches!(sync, Some(SyncStatus::Error(_))) {
            Some(Status::SyncStopped)
        } else if matches!(cloud, CloudState::OtherAccount) {
            Some(Status::SyncPaused)
        } else if plan_limit {
            Some(Status::PlanLimit)
        } else if let Some(progress) = batch.filter(|_| ai_works) {
            Some(Status::Organizing(progress))
        } else if *ai == AiStatus::Off {
            Some(Status::AiOff)
        } else if sync == Some(&SyncStatus::Offline) {
            Some(Status::Offline)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROGRESS: Progress = Progress {
        current: 2,
        total: 5,
        fraction: 0.2,
    };

    fn signed_in(sync: &SyncStatus) -> CloudState<'_> {
        CloudState::SignedIn {
            sync,
            plan_limit: false,
        }
    }

    #[test]
    fn the_most_urgent_message_wins() {
        let attention = AiStatus::NeedsAttention("no credits".into());
        let stopped = SyncStatus::Error("server error".into());
        assert_eq!(
            Status::of(&attention, signed_in(&stopped), Some(PROGRESS)),
            Some(Status::AiNeedsAttention)
        );
        assert_eq!(
            Status::of(&AiStatus::Off, signed_in(&stopped), None),
            Some(Status::SyncStopped)
        );
        let limited = CloudState::SignedIn {
            sync: &SyncStatus::Offline,
            plan_limit: true,
        };
        assert_eq!(
            Status::of(&AiStatus::Ready, limited, Some(PROGRESS)),
            Some(Status::PlanLimit)
        );
        assert_eq!(
            Status::of(
                &AiStatus::Ready,
                signed_in(&SyncStatus::Offline),
                Some(PROGRESS)
            ),
            Some(Status::Organizing(PROGRESS))
        );
        assert_eq!(
            Status::of(&AiStatus::Off, signed_in(&SyncStatus::Offline), None),
            Some(Status::AiOff)
        );
    }

    #[test]
    fn all_well_says_nothing() {
        let up_to_date = SyncStatus::UpToDate;
        assert_eq!(
            Status::of(&AiStatus::Ready, signed_in(&up_to_date), None),
            None
        );
        assert_eq!(Status::of(&AiStatus::Cloud, CloudState::Local, None), None);
    }

    // Work queued for an AI that can't run isn't "organizing": the card says why instead.
    #[test]
    fn a_queue_without_a_working_ai_is_not_organizing() {
        assert_eq!(
            Status::of(&AiStatus::Off, CloudState::Local, Some(PROGRESS)),
            Some(Status::AiOff)
        );
    }
}
