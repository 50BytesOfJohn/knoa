//! The library as the UI sees it: the workspaces, the current workspace's query and its
//! results, sidebar counts, folders and tags. Every database call runs on the background
//! executor.

mod load;

use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};

use gpui::{AppContext as _, Context, EventEmitter, Task};
use knoa_core::{Bookmark, BookmarkId, Folder, FolderTree, Place, Query, Workspace, WorkspaceId};
use knoa_store::{AddedLink, Counts, NewLink, Store};

use crate::state::{
    batch::Batch,
    database::Database,
    facets::{BaseFacets, ResultFacets},
    search::Search,
    settings::SettingsFile,
};
use load::{Base, Loaded, Request};

/// Background refreshes wait at least this long, so a burst of changes shows as one.
const MIN_REFRESH_WAIT: Duration = Duration::from_millis(500);
/// Waiting this many times as long as the last load keeps background reloading to about a
/// tenth of one thread, however large the library.
const LOADS_PER_REFRESH_WAIT: u32 = 10;
/// Any longer and the screen would look stuck while the organizer works.
const MAX_REFRESH_WAIT: Duration = Duration::from_secs(3);

pub struct Library {
    database: Arc<Database>,
    workspace: WorkspaceId,
    workspaces: Vec<Workspace>,
    workspace_sync_ids: BTreeMap<WorkspaceId, String>,
    place: Place,
    search: Search,
    results: Arc<Vec<Bookmark>>,
    result_facets: ResultFacets,
    /// Counts for the search without its typed text, and the query they were taken for.
    base_query: Option<Query>,
    base_facets: Arc<BaseFacets>,
    /// The library changed since `base_facets` were counted.
    base_stale: bool,
    counts: Counts,
    /// The organizing queue's current run, followed through `counts.organizing`.
    batch: Batch,
    folders: FolderTree,
    /// Goes up whenever `folders` changes, so views can keep what they derive from it.
    folders_generation: u64,
    /// The workspace's tags, most used first.
    tags: Vec<(String, usize)>,
    /// The bookmark the detail panel shows, loaded on its own so a change that moves it
    /// out of the results (filing it out of Unsorted) doesn't take it off screen.
    selected: Option<BookmarkId>,
    selected_bookmark: Option<Bookmark>,
    refresh: Option<Task<()>>,
    /// Set while a refresh is waiting out the throttle.
    refresh_pending: bool,
    /// How long the last load took, which sets the throttle.
    last_load: Duration,
}

/// Results changed, so views depending on them should re-render.
pub struct LibraryChanged;

impl EventEmitter<LibraryChanged> for Library {}

/// Which browser profiles feed which workspace changed, so sync has to restart.
pub struct BrowsersChanged;

impl EventEmitter<BrowsersChanged> for Library {}

impl Library {
    pub fn new(database: Arc<Database>, cx: &mut Context<'_, Library>) -> Library {
        let workspace = SettingsFile::get(cx)
            .current_workspace
            .map_or(WorkspaceId(1), WorkspaceId);
        let mut library = Library {
            database,
            workspace,
            workspaces: Vec::new(),
            workspace_sync_ids: BTreeMap::new(),
            place: Place::Everything,
            search: Search::default(),
            results: Arc::default(),
            result_facets: ResultFacets::default(),
            base_query: None,
            base_facets: Arc::default(),
            base_stale: true,
            counts: Counts::default(),
            batch: Batch::default(),
            folders: FolderTree::default(),
            folders_generation: 0,
            tags: Vec::new(),
            selected: None,
            selected_bookmark: None,
            refresh: None,
            refresh_pending: false,
            last_load: Duration::ZERO,
        };
        library.refresh(cx);
        library
    }

    pub fn database(&self) -> Arc<Database> {
        self.database.clone()
    }

    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
    }

    /// The id cloud settings use, once this workspace has been linked.
    pub fn workspace_sync_id(&self, id: WorkspaceId) -> Option<&str> {
        self.workspace_sync_ids.get(&id).map(String::as_str)
    }

    /// The workspace shown. `None` only until the first load finishes.
    pub fn workspace(&self) -> Option<&Workspace> {
        self.workspaces.iter().find(|w| w.id == self.workspace)
    }

    pub fn workspace_id(&self) -> WorkspaceId {
        self.workspace
    }

    pub fn set_workspace(&mut self, id: WorkspaceId, cx: &mut Context<'_, Library>) {
        if self.workspace != id {
            self.workspace = id;
            self.place = Place::Everything;
            // Tags and folders differ per workspace, so the search wouldn't carry over.
            self.search = Search::default();
            SettingsFile::update(cx, |s| s.current_workspace = Some(id.0));
            self.refresh(cx);
        }
    }

    pub fn place(&self) -> Place {
        self.place
    }

    pub fn search(&self) -> &Search {
        &self.search
    }

    pub fn results(&self) -> Arc<Vec<Bookmark>> {
        self.results.clone()
    }

    pub fn result_facets(&self) -> &ResultFacets {
        &self.result_facets
    }

    /// Counts within the search's tokens and place, without its typed text.
    pub fn base_facets(&self) -> &BaseFacets {
        &self.base_facets
    }

    pub fn counts(&self) -> Counts {
        self.counts
    }

    pub fn batch(&self) -> Batch {
        self.batch
    }

    /// The workspace's folders in sidebar order: each followed by its subfolders.
    pub fn folders(&self) -> &[Folder] {
        self.folders.folders()
    }

    /// The workspace's folders as a tree, for depth, paths and moves.
    pub fn folder_tree(&self) -> &FolderTree {
        &self.folders
    }

    /// Changes exactly when [`Library::folder_tree`] does.
    pub fn folders_generation(&self) -> u64 {
        self.folders_generation
    }

    pub fn tags(&self) -> &[(String, usize)] {
        &self.tags
    }

    /// A bookmark in the results, or the selected one wherever it is.
    pub fn bookmark(&self, id: BookmarkId) -> Option<&Bookmark> {
        self.results
            .iter()
            .find(|b| b.id == id)
            .or_else(|| self.selected_bookmark.as_ref().filter(|b| b.id == id))
    }

    /// Keeps `id` loaded through later refreshes, even once it leaves the results.
    pub fn set_selected(&mut self, id: Option<BookmarkId>) {
        if self.selected != id {
            self.selected = id;
            self.selected_bookmark = id.and_then(|id| self.bookmark(id).cloned());
        }
    }

    /// Keeps the search: the place is its scope, one part of it.
    pub fn set_place(&mut self, place: Place, cx: &mut Context<'_, Library>) {
        if self.place != place {
            self.place = place;
            self.refresh(cx);
        }
    }

    pub fn set_search(&mut self, search: Search, cx: &mut Context<'_, Library>) {
        if self.search != search {
            self.search = search;
            self.reload(cx);
        }
    }

    /// Re-runs the query now, for a change to the library. The previous refresh, if still
    /// running, is dropped.
    pub fn refresh(&mut self, cx: &mut Context<'_, Library>) {
        self.base_stale = true;
        self.reload(cx);
    }

    /// Re-runs the query now; the base counts only when they could have changed.
    fn reload(&mut self, cx: &mut Context<'_, Library>) {
        let database = self.database.clone();
        let request = self.request();
        let load = cx.background_spawn(async move {
            let started = Instant::now();
            let loaded = database.read().snapshot(|store| load::load(store, request));
            (loaded, started.elapsed())
        });
        self.refresh_pending = false;
        self.refresh = Some(cx.spawn(async move |this, cx| {
            let (loaded, took) = load.await;
            let _ = this.update(cx, |this, cx| {
                this.last_load = took;
                match loaded {
                    Ok(loaded) => this.apply(loaded, cx),
                    Err(err) => tracing::error!("loading the library failed: {err}"),
                }
            });
        }));
    }

    /// Refreshes at most every half second, and less often when loading takes long, for
    /// bursts of background changes (organizing, imports). User actions call `refresh`
    /// directly.
    pub fn refresh_soon(&mut self, cx: &mut Context<'_, Library>) {
        if self.refresh_pending {
            return;
        }
        self.refresh_pending = true;
        let wait = refresh_wait(self.last_load);
        self.refresh = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(wait).await;
            let _ = this.update(cx, Library::refresh);
        }));
    }

    /// Runs a change against the store off the main thread, then refreshes once it has
    /// committed, so the refresh's reader sees it.
    pub fn change(
        &mut self,
        cx: &mut Context<'_, Library>,
        change: impl FnOnce(&mut Store) -> knoa_store::Result<()> + Send + 'static,
    ) {
        let database = self.database.clone();
        let task = cx.background_spawn(async move { change(&mut database.write()) });
        cx.spawn(async move |this, cx| {
            if let Err(err) = task.await {
                tracing::error!("changing the library failed: {err}");
            }
            let _ = this.update(cx, Library::refresh);
        })
        .detach();
    }

    /// Adds a link by hand to the current workspace and refreshes.
    pub fn add_link(
        &mut self,
        url: String,
        cx: &mut Context<'_, Library>,
    ) -> Task<knoa_store::Result<AddedLink>> {
        let (database, workspace) = (self.database.clone(), self.workspace);
        let now = jiff::Timestamp::now().as_second();
        let task = cx.background_spawn(async move {
            database
                .write()
                .add_link(workspace, &NewLink::new(&url), now)
        });
        cx.spawn(async move |this, cx| {
            let added = task.await;
            let _ = this.update(cx, Library::refresh);
            added
        })
    }

    fn request(&self) -> Request {
        let base_query = self.search.base_query(self.workspace, self.place);
        let base = if self.search.typed.trim().is_empty() {
            Base::Results(base_query)
        } else if !self.base_stale && self.base_query.as_ref() == Some(&base_query) {
            Base::Keep(base_query, self.base_facets.clone())
        } else {
            Base::Search(base_query)
        };
        Request {
            workspace: self.workspace,
            query: self.search.query(self.workspace, self.place),
            selected: self.selected,
            base,
            current: self.results.clone(),
        }
    }

    fn apply(&mut self, loaded: Loaded, cx: &mut Context<'_, Library>) {
        let browsers_changed = browser_map(&self.workspaces) != browser_map(&loaded.workspaces);
        self.workspaces = loaded.workspaces;
        self.workspace_sync_ids = loaded.workspace_sync_ids;
        if browsers_changed {
            cx.emit(BrowsersChanged);
        }
        // A deleted workspace can't stay current.
        if self.workspace().is_none()
            && let Some(first) = self.workspaces.first().map(|w| w.id)
        {
            self.set_workspace(first, cx);
            return;
        }
        self.results = loaded.results;
        self.result_facets = loaded.result_facets;
        self.base_query = Some(loaded.base_query);
        self.base_facets = loaded.base_facets;
        if loaded.base_fresh {
            self.base_stale = false;
        }
        self.counts = loaded.counts;
        self.batch.update(self.counts.organizing);
        if self.folders != loaded.folders {
            self.folders = loaded.folders;
            self.folders_generation += 1;
        }
        self.tags = loaded.tags;
        // The selection may have moved on while this was loading.
        if loaded.selected_id == self.selected {
            self.selected_bookmark = loaded.selected;
        }
        // A folder that was deleted can't stay selected.
        if let Place::Folder(id) = self.place
            && self.folders.get(id).is_none()
        {
            self.set_place(Place::Everything, cx);
        }
        cx.emit(LibraryChanged);
        cx.notify();
    }
}

fn refresh_wait(last_load: Duration) -> Duration {
    last_load
        .saturating_mul(LOADS_PER_REFRESH_WAIT)
        .clamp(MIN_REFRESH_WAIT, MAX_REFRESH_WAIT)
}

fn browser_map(workspaces: &[Workspace]) -> Vec<(WorkspaceId, &[String])> {
    workspaces
        .iter()
        .map(|w| (w.id, w.browsers.as_slice()))
        .collect()
}
