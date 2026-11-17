//! What the sidebar draws from each entity it observes, so that an update it doesn't show,
//! like new search results or another sync round, doesn't rebuild it.
//!
//! The sidebar is a cached view: it draws again only when told to. Whatever it reads in
//! render from these entities has to be in its key here, or it goes stale.

use jiff::Timestamp;
use knoa_core::{Ink, Place, protocol::Plan};

use super::places::PlaceCounts;
use crate::{
    services::cloud::{Cloud, Notice, Phase, SyncStatus},
    state::{batch::Progress, library::Library, settings::Settings},
};

/// From the library: the places, the folders, the workspace button and the batch.
#[derive(PartialEq)]
pub(super) struct LibraryShown {
    place: Place,
    counts: PlaceCounts,
    batch: Option<Progress>,
    folders_generation: u64,
    workspace: Option<(String, Ink)>,
}

impl LibraryShown {
    pub fn of(library: &Library) -> LibraryShown {
        LibraryShown {
            place: library.place(),
            counts: PlaceCounts::of(library.counts()),
            batch: library.batch().progress(),
            folders_generation: library.folders_generation(),
            workspace: library.workspace().map(|w| (w.name.clone(), w.color)),
        }
    }
}

/// From the cloud: the account in the footer and its menu, and what the status card and
/// the menu's sync line say.
#[derive(PartialEq)]
pub(super) struct CloudShown {
    phase: PhaseShown,
    /// The account's email and plan as the footer names it, while signed in.
    account: Option<(Option<String>, Option<Plan>)>,
    sync: SyncShown,
    notices: Vec<Notice>,
    /// When it last synced and the changes waiting, which only the open menu shows.
    sync_line: Option<(Option<Timestamp>, usize)>,
}

#[derive(PartialEq)]
enum PhaseShown {
    SignedOut,
    SigningIn,
    OtherAccount,
    SignedIn,
}

/// The sync status as far as the sidebar tells it apart: starting, syncing and up to
/// date all look the same.
#[derive(PartialEq)]
enum SyncShown {
    Running,
    Offline,
    Stopped,
}

impl CloudShown {
    pub fn of(cloud: &Cloud, settings: &Settings, menu_open: bool) -> CloudShown {
        let phase = match cloud.phase() {
            Phase::SignedOut => PhaseShown::SignedOut,
            Phase::SigningIn(_) => PhaseShown::SigningIn,
            Phase::OtherAccount(_) => PhaseShown::OtherAccount,
            Phase::SignedIn(_) => PhaseShown::SignedIn,
        };
        // As `Identity` picks it: the account last seen until the server answers.
        let account = matches!(phase, PhaseShown::SignedIn | PhaseShown::OtherAccount).then(|| {
            let account = cloud.account().or(settings.account.as_ref());
            (
                account.and_then(|a| a.email.clone()),
                account.map(|a| a.plan),
            )
        });
        let sync = match cloud.sync() {
            SyncStatus::Offline => SyncShown::Offline,
            SyncStatus::Error(_) => SyncShown::Stopped,
            SyncStatus::Starting | SyncStatus::Syncing(_) | SyncStatus::UpToDate => {
                SyncShown::Running
            }
        };
        CloudShown {
            phase,
            account,
            sync,
            notices: cloud.notices(),
            sync_line: menu_open.then(|| (cloud.last_synced(), cloud.waiting_changes())),
        }
    }
}

/// Keeps `now` and says whether it differs from what `last` held.
pub(super) fn update<T: PartialEq>(last: &mut T, now: T) -> bool {
    let changed = *last != now;
    if changed {
        *last = now;
    }
    changed
}
