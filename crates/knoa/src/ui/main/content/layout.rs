//! The masonry layout of the current results, cached until its inputs change.

use std::sync::Arc;

use gpui::Context;
use jiff::{Zoned, civil::Date};
use knoa_core::Bookmark;

use super::GAP;
use crate::ui::main::{
    MainView, cards,
    groups::{self, Group},
    masonry::{Layout, Spec},
};

const HEADER_HEIGHT: f32 = 38.;
const MIN_COLUMN_WIDTH: f32 = 190.;

/// The layout for the current results, recomputed only when its inputs change.
#[derive(Default)]
pub struct LayoutCache {
    key: Option<LayoutKey>,
    pub groups: Vec<Group>,
    pub layout: Layout,
    pub results: Arc<Vec<Bookmark>>,
}

#[derive(PartialEq)]
struct LayoutKey {
    /// The result list's address: the library keeps the same list while its rows don't
    /// change.
    results: usize,
    width: u32,
    columns: usize,
    show_summaries: bool,
    top: u32,
    /// The local date when grouping by date, so the first frame of a new day moves
    /// yesterday's "Today" along. `None` for relevance order, which has no dates.
    today: Option<Date>,
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
        // Relevance order has no dates to group by.
        let by_date = library.search().is_empty();
        let preferred = if show_detail { 3 } else { 4 };
        let columns = preferred
            .min(((width + GAP) / (MIN_COLUMN_WIDTH + GAP)).floor() as usize)
            .max(1);
        let now = by_date.then(Zoned::now);
        let key = LayoutKey {
            results: Arc::as_ptr(&results) as usize,
            width: width.round() as u32,
            columns,
            show_summaries,
            top: top.to_bits(),
            today: now.as_ref().map(Zoned::date),
        };
        if self.layout.key.as_ref() == Some(&key) {
            return;
        }
        let groups = if let Some(now) = &now {
            let added: Vec<i64> = results.iter().map(|b| b.added_at).collect();
            groups::by_date(&added, now)
        } else {
            groups::single("Best matches", results.len())
        };
        let spec = Spec {
            width,
            columns,
            gap: GAP,
            header_height: HEADER_HEIGHT,
            top,
        };
        let layout = Layout::masonry(&groups, &spec, |index, column_width| {
            cards::height(&results[index], column_width, show_summaries)
        });
        self.layout = LayoutCache {
            key: Some(key),
            groups,
            layout,
            results,
        };
    }
}
