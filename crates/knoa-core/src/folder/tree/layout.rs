//! Display order from stored parents, falling back to the top level where they break.

use std::collections::{HashMap, VecDeque};

use crate::{Folder, MAX_FOLDER_DEPTH};

/// The stored parent links, resolved to positions in the input.
pub(super) struct Layout {
    children: Vec<Vec<usize>>,
    roots: Vec<usize>,
}

impl Layout {
    pub(super) fn of(folders: &[Folder]) -> Layout {
        let mut index = HashMap::new();
        for (at, folder) in folders.iter().enumerate() {
            index.entry(folder.id).or_insert(at);
        }
        let mut layout = Layout {
            children: vec![Vec::new(); folders.len()],
            roots: Vec::new(),
        };
        for (at, folder) in folders.iter().enumerate() {
            match folder.parent.and_then(|id| index.get(&id).copied()) {
                Some(parent) if parent != at => layout.children[parent].push(at),
                _ => layout.roots.push(at),
            }
        }
        layout
    }

    /// Each input position with the display parent and depth it gets, in display order.
    /// Folders too deep for their parent, then folders only reachable through a cycle,
    /// start subtrees of their own at the top level.
    pub(super) fn order(&self) -> Vec<(usize, Option<usize>, usize)> {
        let mut walk = Walk {
            layout: self,
            placed: vec![false; self.children.len()],
            order: Vec::with_capacity(self.children.len()),
            too_deep: VecDeque::new(),
        };
        for &root in &self.roots {
            walk.visit(root, None, 0);
        }
        while let Some(at) = walk.next_unplaced() {
            walk.visit(at, None, 0);
        }
        walk.order
    }
}

struct Walk<'a> {
    layout: &'a Layout,
    placed: Vec<bool>,
    order: Vec<(usize, Option<usize>, usize)>,
    too_deep: VecDeque<usize>,
}

impl Walk<'_> {
    fn visit(&mut self, at: usize, parent: Option<usize>, depth: usize) {
        self.placed[at] = true;
        let position = self.order.len();
        self.order.push((at, parent, depth));
        for &child in &self.layout.children[at] {
            if self.placed[child] {
                continue;
            }
            if depth + 1 >= MAX_FOLDER_DEPTH {
                self.too_deep.push_back(child);
            } else {
                self.visit(child, Some(position), depth + 1);
            }
        }
    }

    fn next_unplaced(&mut self) -> Option<usize> {
        while let Some(at) = self.too_deep.pop_front() {
            if !self.placed[at] {
                return Some(at);
            }
        }
        self.placed.iter().position(|placed| !placed)
    }
}
