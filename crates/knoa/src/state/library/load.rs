//! One load of the library from a read snapshot, and the counts computed from it, all on
//! the background executor.

use std::{collections::BTreeMap, sync::Arc};

use knoa_core::{Bookmark, BookmarkId, FolderTree, Query, Workspace, WorkspaceId};
use knoa_store::{Counts, Store};

use crate::state::facets::{BaseFacets, ResultFacets};

/// Enough for any personal library to stay fully browsable; masonry layout of this many
/// cards is a few milliseconds.
pub(super) const RESULT_LIMIT: usize = 20_000;

pub(super) struct Request {
    pub workspace: WorkspaceId,
    pub query: Query,
    pub selected: Option<BookmarkId>,
    pub base: Base,
    /// The results shown now, kept when nothing in them changed.
    pub current: Arc<Vec<Bookmark>>,
}

/// Where the counts suggestions use come from.
pub(super) enum Base {
    /// No typed text: the base query is the query itself.
    Results(Query),
    /// The base query hasn't changed and the library hasn't either.
    Keep(Query, Arc<BaseFacets>),
    /// Run the base query on its own.
    Search(Query),
}

pub(super) struct Loaded {
    pub workspaces: Vec<Workspace>,
    pub workspace_sync_ids: BTreeMap<WorkspaceId, String>,
    pub results: Arc<Vec<Bookmark>>,
    pub result_facets: ResultFacets,
    pub base_query: Query,
    pub base_facets: Arc<BaseFacets>,
    /// The base counts were taken from this load, not kept from an earlier one.
    pub base_fresh: bool,
    pub counts: Counts,
    pub folders: FolderTree,
    pub tags: Vec<(String, usize)>,
    /// The bookmark `selected` was loaded for: `None` there means it no longer exists.
    pub selected_id: Option<BookmarkId>,
    pub selected: Option<Bookmark>,
}

pub(super) fn load(store: &Store, request: Request) -> knoa_store::Result<Loaded> {
    let Request {
        workspace,
        query,
        selected: selected_id,
        base,
        current,
    } = request;
    let selected = match selected_id {
        Some(id) => store.bookmark(id)?,
        None => None,
    };
    let results = same_or_fresh(current, Arc::new(store.search(&query, RESULT_LIMIT)?));
    let folders = store.folders(workspace)?;
    let (base_query, base_facets, base_fresh) = match base {
        Base::Results(base_query) => {
            let facets = BaseFacets::of(&results, &folders);
            (base_query, Arc::new(facets), true)
        }
        Base::Keep(base_query, facets) => (base_query, facets, false),
        Base::Search(base_query) => {
            let found = store.search(&base_query, RESULT_LIMIT)?;
            let facets = BaseFacets::of(&found, &folders);
            (base_query, Arc::new(facets), true)
        }
    };
    Ok(Loaded {
        workspaces: store.workspaces()?,
        workspace_sync_ids: store.workspace_sync_ids()?,
        result_facets: ResultFacets::of(&results),
        results,
        base_query,
        base_facets,
        base_fresh,
        counts: store.counts(workspace)?,
        folders,
        tags: store.tags(workspace)?,
        selected_id,
        selected,
    })
}

/// The list already shown when nothing in it changed, so the card layout, cached by the
/// list's address, survives refreshes that only touched counts, folders or tags.
fn same_or_fresh(current: Arc<Vec<Bookmark>>, fresh: Arc<Vec<Bookmark>>) -> Arc<Vec<Bookmark>> {
    if current == fresh { current } else { fresh }
}
