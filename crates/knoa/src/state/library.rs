//! The library as the UI sees it: the current query and its results, sidebar counts and
//! folders. Every database call runs on the background executor.

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use gpui::{AppContext as _, Context, EventEmitter, Task};
use knoa_core::{Bookmark, BookmarkId, Folder, Kind, Place, Query};
use knoa_store::{Counts, Store};

/// Enough for any personal library to stay fully browsable; masonry layout of this many
/// cards is a few milliseconds.
const RESULT_LIMIT: usize = 20_000;

pub type SharedStore = Arc<Mutex<Store>>;

pub struct Library {
    store: SharedStore,
    place: Place,
    search: String,
    kind: Option<Kind>,
    results: Arc<Vec<Bookmark>>,
    kind_counts: Vec<(Kind, usize)>,
    counts: Counts,
    folders: Vec<Folder>,
    refresh: Option<Task<()>>,
    /// Set while a refresh is waiting out the throttle.
    refresh_pending: bool,
}

/// Results changed, so views depending on them should re-render.
pub struct LibraryChanged;

impl EventEmitter<LibraryChanged> for Library {}

impl Library {
    pub fn new(store: SharedStore, cx: &mut Context<'_, Library>) -> Library {
        let mut library = Library {
            store,
            place: Place::Everything,
            search: String::new(),
            kind: None,
            results: Arc::default(),
            kind_counts: Vec::new(),
            counts: Counts::default(),
            folders: Vec::new(),
            refresh: None,
            refresh_pending: false,
        };
        library.refresh(cx);
        library
    }

    pub fn store(&self) -> SharedStore {
        self.store.clone()
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
        let mut query = Query::parse(&self.search, self.place);
        if self.kind.is_some() {
            query.kind = self.kind;
        }
        query
    }

    /// Re-runs the query now. The previous refresh, if still running, is dropped.
    pub fn refresh(&mut self, cx: &mut Context<'_, Library>) {
        let store = self.store.clone();
        let query = self.query();
        let load = cx.background_spawn(async move { load(&store, &query) });
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

    fn apply(&mut self, loaded: Loaded, cx: &mut Context<'_, Library>) {
        self.results = Arc::new(loaded.results);
        self.kind_counts = loaded.kind_counts;
        self.counts = loaded.counts;
        self.folders = loaded.folders;
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

struct Loaded {
    results: Vec<Bookmark>,
    kind_counts: Vec<(Kind, usize)>,
    counts: Counts,
    folders: Vec<Folder>,
}

fn load(store: &SharedStore, query: &Query) -> knoa_store::Result<Loaded> {
    let store = lock(store);
    Ok(Loaded {
        results: store.search(query, RESULT_LIMIT)?,
        kind_counts: store.kind_counts(query)?,
        counts: store.counts()?,
        folders: store.folders()?,
    })
}

/// A panic while holding the lock leaves the store usable: SQLite transactions roll back
/// on their own.
pub fn lock(store: &SharedStore) -> std::sync::MutexGuard<'_, Store> {
    store
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
