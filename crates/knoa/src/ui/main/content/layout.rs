//! The masonry layout of the current results, cached until its inputs change.

use std::sync::Arc;

use gpui::Context;
use jiff::{Zoned, civil::Date};
use knoa_core::{Bookmark, Place};

use super::GAP;
use crate::ui::main::{
    MainView, cards,
    groups::{self, Group},
    masonry::{Layout, Spec},
    tiles,
};

const HEADER_HEIGHT: f32 = 38.;
const MIN_COLUMN_WIDTH: f32 = 190.;
/// Pictures sit closer together than cards: they have no text to keep apart.
const PICTURE_GAP: f32 = 8.;

/// How the results are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Arrangement {
    /// Cards with their title, site and summary, grouped by date.
    #[default]
    Cards,
    /// The Gallery: pictures only, in one continuous grid. Date headers would break it
    /// into ragged rows of a few pictures each.
    Pictures,
}

/// The layout for the current results, recomputed only when its inputs change.
#[derive(Default)]
pub struct LayoutCache {
    key: Option<LayoutKey>,
    pub arrangement: Arrangement,
    pub groups: Vec<Group>,
    pub layout: Layout,
    pub results: Arc<Vec<Bookmark>>,
}

#[derive(PartialEq)]
struct LayoutKey {
    /// The result list's address: the library keeps the same list while its rows don't
    /// change.
    results: usize,
    arrangement: Arrangement,
    width: u32,
    columns: usize,
    show_summaries: bool,
    top: u32,
    /// The local date when grouping by date, so the first frame of a new day moves
    /// yesterday's "Today" along. `None` for relevance order, which has no dates.
    today: Option<Date>,
}

impl Arrangement {
    fn of(place: Place) -> Arrangement {
        if place == Place::Gallery {
            Arrangement::Pictures
        } else {
            Arrangement::Cards
        }
    }

    /// The gap between items and the height of a group's header.
    fn spacing(self) -> (f32, f32) {
        match self {
            Arrangement::Cards => (GAP, HEADER_HEIGHT),
            Arrangement::Pictures => (PICTURE_GAP, 0.),
        }
    }

    /// Date groups when `now` is given, else one group: labelled for cards, bare for
    /// pictures.
    fn groups(self, results: &[Bookmark], now: Option<&Zoned>) -> Vec<Group> {
        match (now, self) {
            (Some(now), _) => {
                let added: Vec<i64> = results.iter().map(|b| b.added_at).collect();
                groups::by_date(&added, now)
            }
            (None, Arrangement::Cards) => groups::single("Best matches", results.len()),
            (None, Arrangement::Pictures) => groups::single("", results.len()),
        }
    }
}

impl MainView {
    pub(super) fn update_layout(
        &mut self,
        width: f32,
        show_detail: bool,
        top: f32,
        cx: &mut Context<'_, MainView>,
    ) {
        let library = self.library.read(cx);
        let results = library.results();
        let show_summaries = Self::show_summaries(cx);
        let arrangement = Arrangement::of(library.place());
        // Relevance order has no dates to group by.
        let by_date = library.search().is_empty() && arrangement == Arrangement::Cards;
        let preferred = if show_detail { 3 } else { 4 };
        let columns = preferred
            .min(((width + GAP) / (MIN_COLUMN_WIDTH + GAP)).floor() as usize)
            .max(1);
        let now = by_date.then(Zoned::now);
        let key = LayoutKey {
            results: Arc::as_ptr(&results) as usize,
            arrangement,
            width: width.round() as u32,
            columns,
            show_summaries,
            top: top.to_bits(),
            today: now.as_ref().map(Zoned::date),
        };
        if self.layout.key.as_ref() == Some(&key) {
            return;
        }
        let groups = arrangement.groups(&results, now.as_ref());
        let (gap, header_height) = arrangement.spacing();
        let spec = Spec {
            width,
            columns,
            gap,
            header_height,
            top,
        };
        let layout = Layout::masonry(&groups, &spec, |index, column_width| {
            let bookmark = &results[index];
            match arrangement {
                Arrangement::Cards => cards::height(bookmark, column_width, show_summaries),
                Arrangement::Pictures => tiles::height(bookmark, column_width),
            }
        });
        self.layout = LayoutCache {
            key: Some(key),
            arrangement,
            groups,
            layout,
            results,
        };
    }
}
