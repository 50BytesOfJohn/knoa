//! The library as the UI sees it: the workspaces, the current workspace's query and its
//! results, sidebar counts, folders and tags. Every database call runs on the background
//! executor.

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use gpui::{AppContext as _, Context, EventEmitter, Task};
use knoa_core::{Bookmark, BookmarkId, Folder, Kind, Place, Query, Workspace, WorkspaceId};
use knoa_store::{AddedLink, Counts, Store};

use crate::state::settings::SettingsFile;

/// Enough for any personal library to stay fully browsable; masonry layout of this many
/// cards is a few milliseconds.
const RESULT_LIMIT: usize = 20_000;

pub type SharedStore = Arc<Mutex<Store>>;

pub struct Library {
    store: SharedStore,
    workspace: WorkspaceId,
    workspaces: Vec<Workspace>,
    place: Place,
    search: String,
    kind: Option<Kind>,
    results: Arc<Vec<Bookmark>>,
    kind_counts: Vec<(Kind, usize)>,
    counts: Counts,
    folders: Vec<Folder>,
    /// The workspace's tags, most used first.
    tags: Vec<(String, usize)>,
    refresh: Option<Task<()>>,
    /// Set while a refresh is waiting out the throttle.
    refresh_pending: bool,
}

/// Results changed, so views depending on them should re-render.
pub struct LibraryChanged;

impl EventEmitter<LibraryChanged> for Library {}

/// Which browser profiles feed which workspace changed, so sync has to restart.
pub struct BrowsersChanged;

impl EventEmitter<BrowsersChanged> for Library {}

impl Library {
    pub fn new(store: SharedStore, cx: &mut Context<'_, Library>) -> Library {
        let workspace = SettingsFile::get(cx)
            .current_workspace
            .map_or(WorkspaceId(1), WorkspaceId);
        let mut library = Library {
            store,
            workspace,
            workspaces: Vec::new(),
            place: Place::Everything,
            search: String::new(),
            kind: None,
            results: Arc::default(),
            kind_counts: Vec::new(),
            counts: Counts::default(),
            folders: Vec::new(),
            tags: Vec::new(),
            refresh: None,
            refresh_pending: false,
        };
        library.refresh(cx);
        library
    }

    pub fn store(&self) -> SharedStore {
        self.store.clone()
    }

    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
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

    pub fn bookmark(&self, id: BookmarkId) -> Option<&Bookmark> {
        self.results.iter().find(|b| b.id == id)
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
        let store = self.store.clone();
        let query = self.query();
        let workspace = self.workspace;
        let load = cx.background_spawn(async move { load(&store, workspace, &query) });
        self.refresh_pending = false;
        self.refresh = Some(cx.spawn(async move |this, cx| {
            let loaded = load.await;
            let _ = this.update(cx, |this, cx| match loaded {
                Ok(loaded) => this.apply(loaded, cx),
                Err(err) => tracing::error!("loading the library failed: {err}"),
            });
        }));
    }

    /// Refreshes at most twice a second, for bursts of background changes (organizing,
    /// imports). User actions call `refresh` directly.
    pub fn refresh_soon(&mut self, cx: &mut Context<'_, Library>) {
        if self.refresh_pending {
            return;
        }
        self.refresh_pending = true;
        self.refresh = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(500))
                .await;
            let _ = this.update(cx, Library::refresh);
        }));
    }

    /// Runs a change against the store off the main thread, then refreshes.
    pub fn change(
        &mut self,
        cx: &mut Context<'_, Library>,
        change: impl FnOnce(&mut Store) -> knoa_store::Result<()> + Send + 'static,
    ) {
        let store = self.store.clone();
        let task = cx.background_spawn(async move { change(&mut lock(&store)) });
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
        let (store, workspace) = (self.store.clone(), self.workspace);
        let now = jiff::Timestamp::now().as_second();
        let task = cx.background_spawn(async move { lock(&store).add_link(workspace, &url, now) });
        cx.spawn(async move |this, cx| {
            let added = task.await;
            let _ = this.update(cx, Library::refresh);
            added
        })
    }

    fn apply(&mut self, loaded: Loaded, cx: &mut Context<'_, Library>) {
        let browsers_changed = browser_map(&self.workspaces) != browser_map(&loaded.workspaces);
        self.workspaces = loaded.workspaces;
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
        self.results = Arc::new(loaded.results);
        self.kind_counts = loaded.kind_counts;
        self.counts = loaded.counts;
        self.folders = loaded.folders;
        self.tags = loaded.tags;
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

fn browser_map(workspaces: &[Workspace]) -> Vec<(WorkspaceId, &[String])> {
    workspaces
        .iter()
        .map(|w| (w.id, w.browsers.as_slice()))
        .collect()
}

struct Loaded {
    workspaces: Vec<Workspace>,
    results: Vec<Bookmark>,
    kind_counts: Vec<(Kind, usize)>,
    counts: Counts,
    folders: Vec<Folder>,
    tags: Vec<(String, usize)>,
}

fn load(store: &SharedStore, workspace: WorkspaceId, query: &Query) -> knoa_store::Result<Loaded> {
    let store = lock(store);
    Ok(Loaded {
        workspaces: store.workspaces()?,
        results: store.search(query, RESULT_LIMIT)?,
        kind_counts: store.kind_counts(query)?,
        counts: store.counts(workspace)?,
        folders: store.folders(workspace)?,
        tags: store.tags(workspace)?,
    })
}

/// A panic while holding the lock leaves the store usable: SQLite transactions roll back
/// on their own.
pub fn lock(store: &SharedStore) -> std::sync::MutexGuard<'_, Store> {
    store
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
