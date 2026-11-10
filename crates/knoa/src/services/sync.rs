//! Keeps each workspace in step with the browser profiles it chose.
//!
//! Changes arrive through a file watch (FSEvents, debounced), so a new bookmark shows up a
//! couple of seconds after it's saved. A slow periodic rescan backs that up in case an
//! event is missed, for example while the Mac slept. Both skip work when Chromium's
//! checksum hasn't changed.

use std::{collections::HashMap, time::Duration};

use futures::{StreamExt as _, channel::mpsc};
use gpui::{App, AppContext as _, Context, Entity, Global, Task};
use knoa_browser::{Profile, ProfileId};
use knoa_core::WorkspaceId;
use knoa_store::NewBookmarks;

use crate::state::library::{BrowsersChanged, Library};

const RESCAN_EVERY: Duration = Duration::from_mins(10);

/// The running sync, replaced whenever a workspace's browsers change.
/// Holding the entity keeps the sync and its file watch alive.
struct SyncGlobal(#[allow(dead_code)] Option<Entity<BrowserSync>>);

impl Global for SyncGlobal {}

/// Syncs the browsers each workspace chose, restarting whenever that choice changes.
pub fn init(library: &Entity<Library>, cx: &mut App) {
    cx.set_global(SyncGlobal(None));
    cx.subscribe(library, |library, _: &BrowsersChanged, cx| {
        restart(library, cx);
    })
    .detach();
}

fn restart(library: Entity<Library>, cx: &mut App) {
    let mut feeds: HashMap<ProfileId, Vec<WorkspaceId>> = HashMap::new();
    for workspace in library.read(cx).workspaces() {
        for id in workspace
            .browsers
            .iter()
            .filter_map(|id| ProfileId::parse(id))
        {
            feeds.entry(id).or_default().push(workspace.id);
        }
    }
    let sync = (!feeds.is_empty()).then(|| BrowserSync::start(feeds, library, cx));
    cx.set_global(SyncGlobal(sync));
}

pub struct BrowserSync {
    library: Entity<Library>,
    profiles: Vec<Profile>,
    /// The workspaces each profile feeds.
    feeds: HashMap<ProfileId, Vec<WorkspaceId>>,
    checksums: HashMap<ProfileId, String>,
    _watcher: Option<knoa_browser::Watcher>,
    _tasks: Vec<Task<()>>,
}

impl BrowserSync {
    /// Starts syncing each profile into its workspaces, importing once right away.
    fn start(
        feeds: HashMap<ProfileId, Vec<WorkspaceId>>,
        library: Entity<Library>,
        cx: &mut App,
    ) -> Entity<BrowserSync> {
        let profiles: Vec<Profile> = knoa_browser::discover_profiles()
            .into_iter()
            .filter(|p| feeds.contains_key(&p.id))
            .collect();
        cx.new(|cx| {
            let (tx, rx) = mpsc::unbounded::<ProfileId>();
            let watcher = knoa_browser::watch(&profiles, move |id| {
                let _ = tx.unbounded_send(id.clone());
            })
            .inspect_err(|err| tracing::warn!("not watching bookmarks, rescans only: {err}"))
            .ok();
            let tasks = vec![Self::listen(rx, cx), Self::rescan_periodically(cx)];
            let mut sync = BrowserSync {
                library,
                profiles,
                feeds,
                checksums: HashMap::new(),
                _watcher: watcher,
                _tasks: tasks,
            };
            sync.import_all(cx);
            sync
        })
    }

    fn listen(
        mut changes: mpsc::UnboundedReceiver<ProfileId>,
        cx: &mut Context<'_, BrowserSync>,
    ) -> Task<()> {
        cx.spawn(async move |this, cx| {
            while let Some(id) = changes.next().await {
                let _ = this.update(cx, |this, cx| this.import(&id, cx));
            }
        })
    }

    fn rescan_periodically(cx: &mut Context<'_, BrowserSync>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(RESCAN_EVERY).await;
                if this.update(cx, BrowserSync::import_all).is_err() {
                    break;
                }
            }
        })
    }

    pub fn import_all(&mut self, cx: &mut Context<'_, BrowserSync>) {
        let ids: Vec<ProfileId> = self.profiles.iter().map(|p| p.id.clone()).collect();
        for id in ids {
            self.import(&id, cx);
        }
    }

    /// Reads the profile's bookmarks once and imports them into every workspace it feeds.
    fn import(&mut self, id: &ProfileId, cx: &mut Context<'_, BrowserSync>) {
        let Some(profile) = self.profiles.iter().find(|p| &p.id == id) else {
            return;
        };
        let path = profile.bookmarks_path.clone();
        let source = id.to_string();
        let workspaces = self.feeds.get(id).cloned().unwrap_or_default();
        let last_checksum = self.checksums.get(id).cloned();
        let database = self.library.read(cx).database();
        let task = cx.background_spawn(async move {
            let file = knoa_browser::read_bookmarks(&path)?;
            if last_checksum.as_deref() == Some(file.checksum.as_str()) {
                return anyhow::Ok(None);
            }
            let mut store = database.write();
            let mut added = 0;
            for workspace in workspaces {
                let new = if store.has_source(workspace, &source)? {
                    NewBookmarks::Organize
                } else {
                    NewBookmarks::MetadataOnly
                };
                let report = store.import(workspace, &source, &file.bookmarks, new)?;
                tracing::info!(%source, ?workspace, added = report.added, removed = report.removed, "imported bookmarks");
                added += report.added;
            }
            Ok(Some((file.checksum, added)))
        });
        let id = id.clone();
        cx.spawn(async move |this, cx| match task.await {
            Ok(Some((checksum, added))) => {
                let _ = this.update(cx, |this, cx| {
                    this.checksums.insert(id, checksum);
                    this.library.update(cx, Library::refresh);
                    if added > 0 {
                        crate::services::organizer::Organizer::wake(cx);
                    }
                });
            }
            Ok(None) => {}
            Err(err) => tracing::warn!(%id, "bookmark import failed: {err:#}"),
        })
        .detach();
    }
}
