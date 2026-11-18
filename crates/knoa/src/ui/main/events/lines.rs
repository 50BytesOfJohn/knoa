//! A column of headings and event rows at fixed heights, so a long list only builds the
//! rows in view and the keys can scroll to a row without measuring it.

use std::collections::HashSet;

use gpui::{ScrollHandle, SharedString, point, px};

/// One line of a column.
#[derive(Debug, Clone)]
pub enum Line {
    /// A month's name, where the list moves into another month.
    Month(SharedString),
    /// A day, or a group such as "Happening now".
    Heading(SharedString),
    /// An event, by its index in the model.
    Event(usize),
    /// A short sentence where a section has nothing to show.
    Note(SharedString),
}

#[derive(Debug, Clone)]
pub struct Placed {
    pub y: f32,
    pub height: f32,
    pub line: Line,
}

/// The lines of a column, top to bottom.
#[derive(Debug, Default)]
pub struct Lines {
    pub lines: Vec<Placed>,
    pub height: f32,
}

/// The heights of each kind of line in one column.
#[derive(Debug, Clone, Copy)]
pub struct Heights {
    pub month: f32,
    pub heading: f32,
    pub event: f32,
    pub note: f32,
}

impl Lines {
    pub fn push(&mut self, line: Line, heights: &Heights) {
        let height = match line {
            Line::Month(_) => heights.month,
            Line::Heading(_) => heights.heading,
            Line::Event(_) => heights.event,
            Line::Note(_) => heights.note,
        };
        self.lines.push(Placed {
            y: self.height,
            height,
            line,
        });
        self.height += height;
    }

    /// The lines that overlap the band from `top` to `bottom`, each with its position in
    /// the column, which tells apart two rows of the same event.
    pub fn visible(&self, top: f32, bottom: f32) -> impl Iterator<Item = (usize, &Placed)> {
        self.lines
            .iter()
            .enumerate()
            .skip_while(move |(_, placed)| placed.y + placed.height < top)
            .take_while(move |(_, placed)| placed.y <= bottom)
    }

    /// The events in the order the column shows them, each once.
    pub fn order(&self) -> Vec<usize> {
        let mut seen = HashSet::new();
        self.lines
            .iter()
            .filter_map(|placed| match placed.line {
                Line::Event(event) => Some(event),
                _ => None,
            })
            .filter(|event| seen.insert(*event))
            .collect()
    }

    /// Where the first row of `event` is.
    fn row_of(&self, event: usize) -> Option<&Placed> {
        self.lines
            .iter()
            .find(|placed| matches!(placed.line, Line::Event(e) if e == event))
    }

    /// Scrolls `scroll` just enough to bring the first row of `event` into view. The
    /// column starts `origin` down the scrolled content.
    pub fn reveal(&self, event: usize, origin: f32, scroll: &ScrollHandle) {
        let Some(row) = self.row_of(event) else {
            return;
        };
        let viewport = f32::from(scroll.bounds().size.height);
        let top = -f32::from(scroll.offset().y);
        let row_top = origin + row.y;
        let new_top = if row_top < top {
            row_top
        } else if row_top + row.height > top + viewport {
            row_top + row.height - viewport
        } else {
            return;
        };
        scroll.set_offset(point(px(0.), px(-new_top.max(0.))));
    }
}
