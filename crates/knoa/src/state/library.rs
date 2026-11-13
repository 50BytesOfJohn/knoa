//! The library as the UI sees it: the workspaces, the current workspace's query and its
//! results, sidebar counts, folders and tags. Every database call runs on the background
//! executor.

use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};

use gpui::{AppContext as _, Context, EventEmitter, Task};
use knoa_core::{Bookmark, BookmarkId, Folder, Kind, Place, Query, Workspace, WorkspaceId};
use knoa_store::{AddedLink, Counts, Store};

use crate::state::{database::Database, settings::SettingsFile};

/// Enough for any personal library to stay fully browsable; masonry layout of this many
/// cards is a few milliseconds.
const RESULT_LIMIT: usize = 20_000;

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
    search: String,
    kind: Option<Kind>,
    results: Arc<Vec<Bookmark>>,
    kind_counts: Vec<(Kind, usize)>,
    counts: Counts,
    folders: Vec<Folder>,
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
            search: String::new(),
            kind: None,
            results: Arc::default(),
            kind_counts: Vec::new(),
            counts: Counts::default(),
            folders: Vec::new(),
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
            self.kind = None;
            SettingsFile::update(cx, |s| s.current_workspace = Some(id.0));
            self.refresh(cx);
        }
    }

    pub fn place(&self) -> Place {
        self.place
    }

    pub fn search(&self) -> &str {
        &self.search
    }

    pub fn kind(&self) -> Option<Kind> {
        self.kind
    }

    pub fn results(&self) -> Arc<Vec<Bookmark>> {
        self.results.clone()
    }

    pub fn kind_counts(&self) -> &[(Kind, usize)] {
        &self.kind_counts
    }

    pub fn counts(&self) -> Counts {
        self.counts
    }

    pub fn folders(&self) -> &[Folder] {
        &self.folders
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

    pub fn set_place(&mut self, place: Place, cx: &mut Context<'_, Library>) {
        if self.place != place {
            self.place = place;
            self.kind = None;
            self.refresh(cx);
        }
    }

    pub fn set_search(&mut self, text: &str, cx: &mut Context<'_, Library>) {
        if self.search != text {
            text.clone_into(&mut self.search);
            self.refresh(cx);
        }
    }

    pub fn set_kind(&mut self, kind: Option<Kind>, cx: &mut Context<'_, Library>) {
        if self.kind != kind {
            self.kind = kind;
            self.refresh(cx);
        }
    }

    fn query(&self) -> Query {
        let mut query = Query::parse(&self.search, Some(self.workspace), self.place);
        if self.kind.is_some() {
            query.kind = self.kind;
        }
        query
    }

    /// Re-runs the query now. The previous refresh, if still running, is dropped.
    pub fn refresh(&mut self, cx: &mut Context<'_, Library>) {
        let database = self.database.clone();
        let query = self.query();
        let (workspace, selected) = (self.workspace, self.selected);
        let current = self.results.clone();
        let load = cx.background_spawn(async move {
            let started = Instant::now();
            let loaded = database
                .read()
                .snapshot(|store| load(store, workspace, &query, selected));
            let took = started.elapsed();
            let loaded = loaded.map(|loaded| Loaded {
                results: same_or_fresh(current, loaded.results),
                ..loaded
            });
            (loaded, took)
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
        let task =
            cx.background_spawn(async move { database.write().add_link(workspace, &url, now) });
        cx.spawn(async move |this, cx| {
            let added = task.await;
            let _ = this.update(cx, Library::refresh);
            added
        })
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
        self.kind_counts = loaded.kind_counts;
        self.counts = loaded.counts;
        self.folders = loaded.folders;
        self.tags = loaded.tags;
        // The selection may have moved on while this was loading.
        if loaded.selected_id == self.selected {
            self.selected_bookmark = loaded.selected;
        }
        // A folder that was deleted can't stay selected.
        if let Place::Folder(id) = self.place
            && !self.folders.iter().any(|f| f.id == id)
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

/// The list already shown when nothing in it changed, so the card layout, cached by the
/// list's address, survives refreshes that only touched counts, folders or tags.
fn same_or_fresh(current: Arc<Vec<Bookmark>>, fresh: Arc<Vec<Bookmark>>) -> Arc<Vec<Bookmark>> {
    if current == fresh { current } else { fresh }
}

fn browser_map(workspaces: &[Workspace]) -> Vec<(WorkspaceId, &[String])> {
    workspaces
        .iter()
        .map(|w| (w.id, w.browsers.as_slice()))
        .collect()
}

struct Loaded {
    workspaces: Vec<Workspace>,
    workspace_sync_ids: BTreeMap<WorkspaceId, String>,
    results: Arc<Vec<Bookmark>>,
    kind_counts: Vec<(Kind, usize)>,
    counts: Counts,
    folders: Vec<Folder>,
    tags: Vec<(String, usize)>,
    /// The bookmark `selected` was loaded for: `None` there means it no longer exists.
    selected_id: Option<BookmarkId>,
    selected: Option<Bookmark>,
}

fn load(
    store: &Store,
    workspace: WorkspaceId,
    query: &Query,
    selected_id: Option<BookmarkId>,
) -> knoa_store::Result<Loaded> {
    let selected = match selected_id {
        Some(id) => store.bookmark(id)?,
        None => None,
    };
    Ok(Loaded {
        workspaces: store.workspaces()?,
        workspace_sync_ids: store.workspace_sync_ids()?,
        results: Arc::new(store.search(query, RESULT_LIMIT)?),
        kind_counts: store.kind_counts(query)?,
        counts: store.counts(workspace)?,
        folders: store.folders(workspace)?,
        tags: store.tags(workspace)?,
        selected_id,
        selected,
    })
}
