//! The places at the top of the sidebar: Everything, Gallery, Events, Up next and the rest.

use std::rc::Rc;

use gpui::{
    Animation, AnimationExt as _, AnyElement, Context, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, Radians, SharedString, StatefulInteractiveElement as _,
    Styled as _, Transformation, div,
};
use knoa_core::{Language, Place};
use knoa_store::Counts;

use super::{
    LIST_TOP, SidebarEvent, SidebarView,
    row::{ICON, ROW_GAP, ROW_HEIGHT, count, label, row_frame},
};
use crate::{
    assets::line_icon,
    i18n::{self, t},
    services::organizer::AiStatus,
    ui::{components::svg_icon, motion, theme::Theme},
};

/// One place as the sidebar draws it, made when the counts change rather than per frame.
struct PlaceRow {
    place: Place,
    label: SharedString,
    icon: SharedString,
    count: SharedString,
}

/// The places shown for the current counts, kept until the counts, the place or the
/// language change.
#[derive(Default)]
pub(super) struct PlaceRows {
    made_for: Option<(PlaceCounts, Place, Language)>,
    rows: Rc<[PlaceRow]>,
}

/// The library's counts as the places show them: all but `organizing`, which counts the
/// queue across every workspace and is followed through the batch instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PlaceCounts(Counts);

impl PlaceCounts {
    pub fn of(counts: Counts) -> PlaceCounts {
        PlaceCounts(Counts {
            organizing: 0,
            ..counts
        })
    }
}

impl PlaceRows {
    fn rows(&mut self, counts: PlaceCounts, current: Place) -> Rc<[PlaceRow]> {
        let made_for = Some((counts, current, i18n::language()));
        if self.made_for != made_for {
            self.rows = visible_places(counts.0, current).into();
            self.made_for = made_for;
        }
        self.rows.clone()
    }

    /// How far down the list's content the row of `place` ends, while it is shown.
    fn bottom(&self, place: Place) -> Option<Pixels> {
        let index = self.rows.iter().position(|row| row.place == place)?;
        Some(LIST_TOP + ROW_HEIGHT * (index + 1) as f32 + ROW_GAP * index as f32)
    }
}

fn visible_places(counts: Counts, current: Place) -> Vec<PlaceRow> {
    let places = [
        (
            Place::Everything,
            t!("sidebar-place-everything"),
            line_icon!("layers"),
            counts.everything,
        ),
        (
            Place::Gallery,
            t!("sidebar-place-gallery"),
            line_icon!("image"),
            counts.gallery,
        ),
        (
            Place::Events,
            t!("sidebar-place-events"),
            line_icon!("calendar"),
            counts.events,
        ),
        (
            Place::UpNext,
            t!("sidebar-place-up-next"),
            line_icon!("bookmark"),
            counts.up_next,
        ),
        (
            Place::Unsorted,
            t!("sidebar-place-unsorted"),
            line_icon!("inbox"),
            counts.unsorted,
        ),
        (
            Place::NotOrganized,
            t!("sidebar-place-not-organized"),
            line_icon!("circle-dashed"),
            counts.not_organized,
        ),
        (
            Place::SetAside,
            t!("sidebar-place-set-aside"),
            line_icon!("archive"),
            counts.set_aside,
        ),
        (
            Place::Trash,
            t!("sidebar-place-trash"),
            line_icon!("trash"),
            counts.trash,
        ),
    ];
    places
        .into_iter()
        // These show only while they hold something, or while they are open.
        .filter(|&(place, _, _, n)| {
            let hideable = matches!(
                place,
                Place::Gallery | Place::Events | Place::NotOrganized | Place::SetAside | Place::Trash
            );
            !hideable || n > 0 || place == current
        })
        .map(|(place, label, icon, n)| PlaceRow {
            place,
            label: label.into(),
            icon,
            count: match n {
                0 => SharedString::default(),
                n => SharedString::from(n.to_string()),
            },
        })
        .collect()
}

impl SidebarView {
    pub(super) fn place_list(
        &mut self,
        current: Place,
        cx: &mut Context<'_, SidebarView>,
    ) -> gpui::Div {
        let library = self.library.read(cx);
        let (counts, organizing) = (library.counts(), library.batch().progress().is_some());
        let organizing = organizing
            && matches!(
                self.organizer.read(cx).status(),
                AiStatus::Ready | AiStatus::Cloud
            );
        let rows = self.places.rows(PlaceCounts::of(counts), current);
        let in_view = self.not_organized_in_view();
        let turning = organizing && in_view;
        let mut list = div().flex().flex_col().gap(ROW_GAP);
        for (index, row) in rows.iter().enumerate() {
            let selected = row.place == current;
            list = list.child(place_row(row, index, selected, turning, cx));
        }
        if organizing {
            list = self.recheck_in_view_after_layout(list, in_view, cx);
        }
        list
    }

    /// Whether any of Not organized's row shows in the list. A turning animation asks for
    /// the next frame even while scrolled out of sight, so out of sight it stands still.
    /// GPUI redraws the sidebar whenever the list scrolls, which keeps this current.
    fn not_organized_in_view(&self) -> bool {
        let Some(bottom) = self.places.bottom(Place::NotOrganized) else {
            return false;
        };
        let scroll = &self.drop.scroll;
        // Until layout clamps it, the offset can be past the end of a list that shrank.
        let scrolled = (-scroll.offset().y).clamp(Pixels::ZERO, scroll.max_offset().y);
        bottom > scrolled
    }

    /// Redraws the sidebar on the next frame when layout moved Not organized into or out
    /// of view after `rendered` was decided. Layout clamps the scroll offset when the list
    /// shrinks or grows taller, without redrawing the sidebar.
    fn recheck_in_view_after_layout(
        &self,
        list: gpui::Div,
        rendered: bool,
        cx: &mut Context<'_, SidebarView>,
    ) -> gpui::Div {
        let sidebar = cx.weak_entity();
        // The places' children are prepainted after the list has clamped its offset.
        list.on_children_prepainted(move |_, window, cx| {
            let Ok(in_view) = sidebar.read_with(cx, |sidebar, _| sidebar.not_organized_in_view())
            else {
                return;
            };
            if in_view != rendered {
                let sidebar = sidebar.clone();
                window.on_next_frame(move |_, cx| {
                    let _ = sidebar.update(cx, |_, cx| cx.notify());
                });
            }
        })
    }
}

fn place_row(
    row: &PlaceRow,
    index: usize,
    selected: bool,
    turning: bool,
    cx: &mut Context<'_, SidebarView>,
) -> AnyElement {
    let theme = Theme::get(cx);
    let place = row.place;
    let color = if selected { theme.accent } else { theme.muted };
    let icon = svg_icon(row.icon.clone(), ICON, color);
    let icon = if place == Place::NotOrganized && turning {
        turning_icon(icon)
    } else {
        icon.into_any_element()
    };
    row_frame(selected, theme)
        .id(("place", index))
        .child(icon)
        .child(label(row.label.clone()))
        .child(count(row.count.clone(), theme))
        .on_click(cx.listener(move |_, _, _, cx| cx.emit(SidebarEvent::Show(place))))
        .into_any_element()
}

/// The dashed circle turning slowly while the AI works. Still with Reduce Motion, which
/// GPUI's animations follow.
fn turning_icon(icon: gpui::Svg) -> AnyElement {
    let turn = Animation::new(motion::SLOW_TURN)
        .repeat()
        .with_max_fps(motion::LOOP_FPS);
    icon.with_animation("not-organized-turning", turn, |icon, delta| {
        icon.with_transformation(Transformation::rotate(Radians(
            delta * std::f32::consts::TAU,
        )))
    })
    .into_any_element()
}
