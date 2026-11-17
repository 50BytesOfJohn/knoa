//! The places at the top of the sidebar: Everything, Gallery, Up next and the rest.

use std::rc::Rc;

use gpui::{
    Animation, AnimationExt as _, AnyElement, Context, InteractiveElement as _, IntoElement,
    ParentElement as _, Radians, SharedString, StatefulInteractiveElement as _, Styled as _,
    Transformation, div,
};
use knoa_core::Place;
use knoa_store::Counts;

use super::{
    SidebarEvent, SidebarView,
    row::{ICON, ROW_GAP, count, label, row_frame},
};
use crate::{
    assets::line_icon,
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

/// The places shown for the current counts, kept until the counts or the place change.
#[derive(Default)]
pub(super) struct PlaceRows {
    made_for: Option<(Counts, Place)>,
    rows: Rc<[PlaceRow]>,
}

impl PlaceRows {
    fn rows(&mut self, counts: Counts, current: Place) -> Rc<[PlaceRow]> {
        if self.made_for != Some((counts, current)) {
            self.rows = visible_places(counts, current).into();
            self.made_for = Some((counts, current));
        }
        self.rows.clone()
    }
}

fn visible_places(counts: Counts, current: Place) -> Vec<PlaceRow> {
    let places = [
        (
            Place::Everything,
            "Everything",
            line_icon!("layers"),
            counts.everything,
        ),
        (
            Place::Gallery,
            "Gallery",
            line_icon!("image"),
            counts.gallery,
        ),
        (
            Place::UpNext,
            "Up next",
            line_icon!("bookmark"),
            counts.up_next,
        ),
        (
            Place::Unsorted,
            "Unsorted",
            line_icon!("inbox"),
            counts.unsorted,
        ),
        (
            Place::NotOrganized,
            "Not organized",
            line_icon!("circle-dashed"),
            counts.not_organized,
        ),
        (
            Place::SetAside,
            "Set aside",
            line_icon!("archive"),
            counts.set_aside,
        ),
        (Place::Trash, "Trash", line_icon!("trash"), counts.trash),
    ];
    places
        .into_iter()
        // These show only while they hold something, or while they are open.
        .filter(|&(place, _, _, n)| {
            let hideable = matches!(
                place,
                Place::Gallery | Place::NotOrganized | Place::SetAside | Place::Trash
            );
            !hideable || n > 0 || place == current
        })
        .map(|(place, label, icon, n)| PlaceRow {
            place,
            label: SharedString::new_static(label),
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
        let rows = self.places.rows(counts, current);
        let mut list = div().flex().flex_col().gap(ROW_GAP);
        for (index, row) in rows.iter().enumerate() {
            let selected = row.place == current;
            list = list.child(place_row(row, index, selected, organizing, cx));
        }
        list
    }
}

fn place_row(
    row: &PlaceRow,
    index: usize,
    selected: bool,
    organizing: bool,
    cx: &mut Context<'_, SidebarView>,
) -> AnyElement {
    let theme = Theme::get(cx);
    let place = row.place;
    let color = if selected { theme.accent } else { theme.muted };
    let icon = svg_icon(row.icon.clone(), ICON, color);
    let icon = if place == Place::NotOrganized && organizing {
        turning(icon)
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
fn turning(icon: gpui::Svg) -> AnyElement {
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
