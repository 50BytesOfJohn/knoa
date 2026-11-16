//! A workspace's folders as the tree the user sees, built defensively from stored parents.

use std::{collections::HashMap, fmt, ops::Range};

mod layout;

use layout::Layout;

use super::{Folder, FolderId, MAX_FOLDER_DEPTH};

/// What separates the names in a folder's path, as in "Wallpapers › Anime".
const PATH_SEPARATOR: &str = " › ";

/// One workspace's folders in display order: each folder followed by its subfolders, and
/// siblings in the order they were given. A folder whose parent is missing, in a cycle, or
/// would sit deeper than [`MAX_FOLDER_DEPTH`] shows at the top level, so any rows make a
/// valid tree.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FolderTree {
    folders: Vec<Folder>,
    nodes: Vec<Node>,
    index: HashMap<FolderId, usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Node {
    parent: Option<usize>,
    depth: usize,
    /// One past the last folder of its subtree, which is contiguous in display order.
    end: usize,
}

/// Why a folder can't go where it was asked to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    /// The parent isn't one of this workspace's folders.
    UnknownParent,
    /// The parent is the folder itself or one of its subfolders.
    IntoItself,
    /// The folder or its subfolders would sit deeper than [`MAX_FOLDER_DEPTH`].
    TooDeep,
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MoveError::UnknownParent => "the parent folder isn't in this workspace",
            MoveError::IntoItself => "a folder can't go inside itself",
            MoveError::TooDeep => "folders nest at most three levels deep",
        })
    }
}

impl std::error::Error for MoveError {}

impl FolderTree {
    /// Builds the tree. `folders` gives sibling order: the store passes them by position.
    pub fn new(folders: Vec<Folder>) -> FolderTree {
        let order = Layout::of(&folders).order();
        let mut slots: Vec<Option<Folder>> = folders.into_iter().map(Some).collect();
        let mut tree = FolderTree::default();
        for (input, parent, depth) in order {
            let folder = slots[input].take().expect("each folder is placed once");
            tree.index.insert(folder.id, tree.folders.len());
            tree.folders.push(folder);
            tree.nodes.push(Node {
                parent,
                depth,
                end: 0,
            });
        }
        tree.close_subtrees();
        tree
    }

    /// The same tree with each folder's [`Folder::count`] set by `count`.
    #[must_use]
    pub fn with_counts(mut self, count: impl Fn(FolderId) -> usize) -> FolderTree {
        for folder in &mut self.folders {
            folder.count = count(folder.id);
        }
        self
    }

    /// Every folder, in display order.
    pub fn folders(&self) -> &[Folder] {
        &self.folders
    }

    pub fn len(&self) -> usize {
        self.folders.len()
    }

    pub fn is_empty(&self) -> bool {
        self.folders.is_empty()
    }

    /// Whether any folder has a parent. A flat tree shows no hierarchy chrome.
    pub fn is_nested(&self) -> bool {
        self.nodes.iter().any(|node| node.depth > 0)
    }

    pub fn get(&self, id: FolderId) -> Option<&Folder> {
        self.index.get(&id).map(|&at| &self.folders[at])
    }

    /// 0 for a top-level folder, and for one that isn't here.
    pub fn depth(&self, id: FolderId) -> usize {
        self.node(id).map_or(0, |node| node.depth)
    }

    /// The parent it shows under, which may differ from the stored one.
    pub fn parent(&self, id: FolderId) -> Option<&Folder> {
        let parent = self.node(id)?.parent?;
        Some(&self.folders[parent])
    }

    /// The folders directly under `parent`, or the top-level ones for `None`.
    pub fn children(&self, parent: Option<FolderId>) -> impl Iterator<Item = &Folder> {
        let (range, depth) = match parent.and_then(|id| self.index.get(&id)) {
            Some(&at) => (at + 1..self.nodes[at].end, self.nodes[at].depth + 1),
            None if parent.is_some() => (0..0, 0),
            None => (0..self.len(), 0),
        };
        self.range(range)
            .filter(move |(_, node)| node.depth == depth)
            .map(|(folder, _)| folder)
    }

    /// The folder followed by everything under it. Empty when it isn't here.
    pub fn subtree(&self, id: FolderId) -> &[Folder] {
        match self.index.get(&id) {
            Some(&at) => &self.folders[at..self.nodes[at].end],
            None => &[],
        }
    }

    /// Everything under the folder, without it.
    pub fn descendants(&self, id: FolderId) -> &[Folder] {
        self.subtree(id).get(1..).unwrap_or_default()
    }

    /// Levels in the folder's subtree: 1 for a folder with no subfolders.
    pub fn height(&self, id: FolderId) -> usize {
        let Some(&at) = self.index.get(&id) else {
            return 0;
        };
        let base = self.nodes[at].depth;
        self.nodes[at..self.nodes[at].end]
            .iter()
            .map(|node| node.depth - base + 1)
            .max()
            .unwrap_or(1)
    }

    /// The top-level folder down to this one. Empty when it isn't here.
    pub fn path(&self, id: FolderId) -> Vec<&Folder> {
        let mut path = Vec::new();
        let mut at = self.index.get(&id).copied();
        while let Some(index) = at {
            path.push(&self.folders[index]);
            at = self.nodes[index].parent;
        }
        path.reverse();
        path
    }

    /// The folder's name with its parents' before it, "Wallpapers › Anime", for places
    /// that list folders flat.
    pub fn path_name(&self, id: FolderId) -> String {
        let names: Vec<&str> = self.path(id).iter().map(|f| f.name.as_str()).collect();
        names.join(PATH_SEPARATOR)
    }

    /// Whether a new folder can go under `parent`.
    ///
    /// # Errors
    ///
    /// When the parent isn't here or is already at the deepest level.
    pub fn check_new_child(&self, parent: Option<FolderId>) -> Result<(), MoveError> {
        self.check_room(parent, 1)
    }

    /// Whether `folder` and its subfolders can move under `parent`.
    ///
    /// # Errors
    ///
    /// When the parent isn't here, is inside the folder, or the subtree wouldn't fit.
    pub fn check_move(&self, folder: FolderId, parent: Option<FolderId>) -> Result<(), MoveError> {
        if let Some(parent) = parent
            && self.subtree(folder).iter().any(|f| f.id == parent)
        {
            return Err(MoveError::IntoItself);
        }
        self.check_room(parent, self.height(folder).max(1))
    }

    fn check_room(&self, parent: Option<FolderId>, height: usize) -> Result<(), MoveError> {
        let depth = match parent {
            Some(id) => self.node(id).ok_or(MoveError::UnknownParent)?.depth + 1,
            None => 0,
        };
        if depth + height > MAX_FOLDER_DEPTH {
            return Err(MoveError::TooDeep);
        }
        Ok(())
    }

    fn node(&self, id: FolderId) -> Option<&Node> {
        self.index.get(&id).map(|&at| &self.nodes[at])
    }

    fn range(&self, range: Range<usize>) -> impl Iterator<Item = (&Folder, &Node)> {
        self.folders[range.clone()].iter().zip(&self.nodes[range])
    }

    /// Sets each node's `end`: its subtree runs until the next folder at its depth or above.
    fn close_subtrees(&mut self) {
        let mut open: Vec<usize> = Vec::new();
        for at in 0..self.nodes.len() {
            let depth = self.nodes[at].depth;
            while let Some(&last) = open.last()
                && self.nodes[last].depth >= depth
            {
                self.nodes[last].end = at;
                open.pop();
            }
            open.push(at);
        }
        for at in open {
            self.nodes[at].end = self.nodes.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FolderColor;

    fn folder(id: i64, parent: Option<i64>) -> Folder {
        Folder {
            id: FolderId(id),
            parent: parent.map(FolderId),
            name: format!("f{id}"),
            description: String::new(),
            auto_assign: true,
            color: FolderColor::default(),
            icon: None,
            count: 0,
        }
    }

    fn shown(tree: &FolderTree) -> Vec<(i64, usize)> {
        tree.folders()
            .iter()
            .map(|f| (f.id.0, tree.depth(f.id)))
            .collect()
    }

    #[test]
    fn children_follow_their_parent_whatever_order_the_rows_come_in() {
        let tree = FolderTree::new(vec![
            folder(3, Some(2)),
            folder(4, None),
            folder(2, Some(1)),
            folder(1, None),
            folder(5, Some(1)),
        ]);

        assert_eq!(shown(&tree), [(4, 0), (1, 0), (2, 1), (3, 2), (5, 1)]);
        assert_eq!(tree.path_name(FolderId(3)), "f1 › f2 › f3");
        let ids = |folders: &[Folder]| folders.iter().map(|f| f.id.0).collect::<Vec<_>>();
        assert_eq!(ids(tree.subtree(FolderId(1))), [1, 2, 3, 5]);
        let children: Vec<i64> = tree.children(Some(FolderId(1))).map(|f| f.id.0).collect();
        assert_eq!(children, [2, 5]);
        let top: Vec<i64> = tree.children(None).map(|f| f.id.0).collect();
        assert_eq!(top, [4, 1]);
    }

    #[test]
    fn missing_cyclic_and_too_deep_parents_fall_back_to_the_top_level() {
        let tree = FolderTree::new(vec![
            folder(1, Some(99)),
            folder(2, Some(3)),
            folder(3, Some(2)),
            folder(4, None),
            folder(5, Some(4)),
            folder(6, Some(5)),
            folder(7, Some(6)),
            folder(8, Some(8)),
        ]);

        assert_eq!(tree.len(), 8, "every folder is shown once");
        for id in [1, 4, 7, 8] {
            assert_eq!(tree.parent(FolderId(id)), None, "folder {id}");
        }
        assert_eq!(tree.depth(FolderId(6)), 2);
        let cycle = [2, 3].map(|id| tree.depth(FolderId(id)));
        assert!(cycle.contains(&0) && cycle.contains(&1), "{cycle:?}");
    }

    #[test]
    fn a_move_keeps_the_whole_subtree_within_the_depth_limit() {
        let tree = FolderTree::new(vec![
            folder(1, None),
            folder(2, Some(1)),
            folder(3, Some(2)),
            folder(4, None),
            folder(5, Some(4)),
        ]);
        let move_to = |id, parent: Option<i64>| tree.check_move(FolderId(id), parent.map(FolderId));

        assert_eq!(move_to(1, Some(3)), Err(MoveError::IntoItself));
        assert_eq!(move_to(4, Some(2)), Err(MoveError::TooDeep));
        assert_eq!(move_to(4, Some(1)), Ok(()));
        assert_eq!(move_to(4, Some(99)), Err(MoveError::UnknownParent));
        assert_eq!(move_to(5, Some(2)), Ok(()));
        assert_eq!(move_to(2, None), Ok(()));
        assert_eq!(
            tree.check_new_child(Some(FolderId(3))),
            Err(MoveError::TooDeep)
        );
        assert_eq!(tree.check_new_child(Some(FolderId(2))), Ok(()));
    }
}
