//! What the sidebar's status card says: the most urgent thing the AI or sync has to tell,
//! or nothing while all is well.

use crate::{
    services::{
        cloud::{Notice, SyncStatus},
        organizer::AiStatus,
    },
    state::batch::Progress,
};

/// One message for the card, most urgent first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Status {
    AiNeedsAttention,
    /// The cloud organizes, and this month's allowance for it is used up.
    AllowanceUsedUp,
    SyncStopped,
    /// Signed in, but the library is linked to another account, so nothing syncs.
    SyncPaused,
    /// The server refused some changes, which stayed on this Mac.
    ChangesNotSynced,
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
        notices: &'a [Notice],
    },
}

impl Status {
    /// The message to show, if any. Sync speaks only while signed in.
    pub fn of(ai: &AiStatus, cloud: CloudState<'_>, batch: Option<Progress>) -> Option<Status> {
        let (sync, notices) = match cloud {
            CloudState::SignedIn { sync, notices } => (Some(sync), notices),
            CloudState::Local | CloudState::OtherAccount => (None, &[][..]),
        };
        let noticed = |wanted: fn(&Notice) -> bool| notices.iter().any(wanted);
        let ai_works = matches!(ai, AiStatus::Ready | AiStatus::Cloud);
        if matches!(ai, AiStatus::NeedsAttention(_)) {
            Some(Status::AiNeedsAttention)
        } else if noticed(|n| *n == Notice::AllowanceUsedUp) {
            Some(Status::AllowanceUsedUp)
        } else if matches!(sync, Some(SyncStatus::Error(_))) {
            Some(Status::SyncStopped)
        } else if matches!(cloud, CloudState::OtherAccount) {
            Some(Status::SyncPaused)
        } else if noticed(|n| matches!(n, Notice::Rejected(_))) {
            Some(Status::ChangesNotSynced)
        } else if noticed(|n| matches!(n, Notice::PlanLimit { .. })) {
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
