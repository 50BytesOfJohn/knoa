//! Which folders the sidebar shows: the tree minus what is collapsed under a closed
//! parent. The rows are worked out once per change to the folders or to what is
//! collapsed, not on every frame.

use std::{collections::BTreeSet, rc::Rc};

use gpui::{App, SharedString};
use knoa_core::{FolderColor, FolderIcon, FolderId, FolderTree};

use crate::state::settings::SettingsFile;

/// One visible folder, with all the sidebar draws for it.
#[derive(Debug, Clone)]
pub(super) struct FolderRow {
    pub id: FolderId,
    pub name: SharedString,
    pub icon: Option<FolderIcon>,
    pub color: FolderColor,
    /// The bookmark count as shown, empty for none.
    pub count: SharedString,
    pub depth: usize,
    pub disclosure: Disclosure,
}

/// Whether a row has subfolders, and whether they show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Disclosure {
    Leaf,
    Expanded,
    Collapsed,
}

/// The collapsed parents, kept in settings on this Mac, and the rows they leave visible.
pub(in crate::ui::main) struct Outline {
    collapsed: BTreeSet<FolderId>,
    /// Goes up whenever `collapsed` changes.
    generation: u64,
    /// The library's folder generation and ours the rows were made for.
    made_for: Option<(u64, u64)>,
    rows: Rc<[Rc<FolderRow>]>,
    nested: bool,
}

impl Outline {
    pub fn load(cx: &App) -> Outline {
        let collapsed = SettingsFile::get(cx)
            .collapsed_folders
            .iter()
            .map(|&id| FolderId(id))
            .collect();
        Outline {
            collapsed,
            generation: 0,
            made_for: None,
            rows: Rc::from([]),
            nested: false,
        }
    }

    /// The visible rows for `tree`, remade only when it or the collapsed set changed. Each
    /// row is shared, so the elements that hold one copy a pointer.
    pub(super) fn rows(&mut self, tree: &FolderTree, tree_generation: u64) -> Rc<[Rc<FolderRow>]> {
        let key = (tree_generation, self.generation);
        if self.made_for != Some(key) {
            self.rows = visible_rows(tree, &self.collapsed).into();
            self.nested = tree.is_nested();
            self.made_for = Some(key);
        }
        self.rows.clone()
    }

    /// Whether any folder has a parent, as of the last [`Outline::rows`].
    pub(super) fn is_nested(&self) -> bool {
        self.nested
    }

    /// The `n`th visible folder, counting from 0, as ⌘1 to ⌘9 pick them.
    pub fn nth(&mut self, tree: &FolderTree, tree_generation: u64, n: usize) -> Option<FolderId> {
        self.rows(tree, tree_generation).get(n).map(|row| row.id)
    }

    pub fn toggle(&mut self, id: FolderId, cx: &mut App) {
        if !self.collapsed.remove(&id) {
            self.collapsed.insert(id);
        }
        self.changed(cx);
    }

    /// Shows the folder's children.
    pub fn expand(&mut self, id: FolderId, cx: &mut App) {
        if self.collapsed.remove(&id) {
            self.changed(cx);
        }
    }

    /// Opens each of `parents`, so whatever is under them is visible.
    pub fn reveal(&mut self, parents: &[FolderId], cx: &mut App) {
        let before = self.collapsed.len();
        for id in parents {
            self.collapsed.remove(id);
        }
        if self.collapsed.len() != before {
            self.changed(cx);
        }
    }

    fn changed(&mut self, cx: &mut App) {
        self.generation += 1;
        let ids = self.collapsed.iter().map(|id| id.0).collect();
        SettingsFile::update(cx, |s| s.collapsed_folders = ids);
    }
}

/// Every folder in display order, skipping the subtrees of collapsed parents.
fn visible_rows(tree: &FolderTree, collapsed: &BTreeSet<FolderId>) -> Vec<Rc<FolderRow>> {
    let folders = tree.folders();
    let mut rows = Vec::with_capacity(folders.len());
    let mut at = 0;
    while let Some(folder) = folders.get(at) {
        let below = tree.descendants(folder.id).len();
        let disclosure = match (below, collapsed.contains(&folder.id)) {
            (0, _) => Disclosure::Leaf,
            (_, true) => Disclosure::Collapsed,
            (_, false) => Disclosure::Expanded,
        };
        rows.push(Rc::new(FolderRow {
            id: folder.id,
            name: SharedString::from(folder.name.clone()),
            icon: folder.icon.clone(),
            color: folder.color,
            count: match folder.count {
                0 => SharedString::default(),
                n => SharedString::from(n.to_string()),
            },
            depth: tree.depth(folder.id),
            disclosure,
        }));
        at += if disclosure == Disclosure::Collapsed {
            below + 1
        } else {
            1
        };
    }
    rows
}
