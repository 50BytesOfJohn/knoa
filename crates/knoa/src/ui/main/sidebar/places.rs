//! The places at the top of the sidebar: Everything, Gallery, Up next and the rest.

use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _,
    px,
};
use knoa_core::Place;

use super::{ROW_INSET, ROW_PADDING, SidebarEvent, SidebarView};
use crate::ui::{
    components::icon,
    mascot::ai_mark,
    theme::{ROW_HEIGHT, ROW_RADIUS, Theme},
};

/// What leads a place's row: a Lucide icon, or the AI's dot for places about the AI.
#[derive(Clone, Copy)]
enum PlaceMark {
    Icon(&'static str),
    Ai,
}

impl SidebarView {
    pub(super) fn places(
        &self,
        counts: knoa_store::Counts,
        current: Place,
        cx: &mut Context<'_, SidebarView>,
    ) -> gpui::Div {
        let places = [
            (
                Place::Everything,
                "Everything",
                PlaceMark::Icon("layers"),
                counts.everything,
            ),
            (
                Place::Gallery,
                "Gallery",
                PlaceMark::Icon("images"),
                counts.gallery,
            ),
            (
                Place::UpNext,
                "Up next",
                PlaceMark::Icon("bookmark"),
                counts.up_next,
            ),
            (
                Place::Unsorted,
                "Unsorted",
                PlaceMark::Icon("inbox"),
                counts.unsorted,
            ),
            (
                Place::NotOrganized,
                "Not organized",
                PlaceMark::Ai,
                counts.not_organized,
            ),
            (
                Place::SetAside,
                "Set aside",
                PlaceMark::Icon("archive"),
                counts.set_aside,
            ),
            (
                Place::Trash,
                "Trash",
                PlaceMark::Icon("trash"),
                counts.trash,
            ),
        ];
        let mut list = div().flex().flex_col().gap_0p5().px(ROW_INSET);
        for (place, label, mark, count) in places {
            // The Gallery, Set aside and Trash only show while they hold something.
            let hideable = matches!(place, Place::Gallery | Place::SetAside | Place::Trash);
            if hideable && count == 0 && current != place {
                continue;
            }
            list = list.child(self.place_row(place, current, label, mark, count, cx));
        }
        list
    }

    fn place_row(
        &self,
        place: Place,
        current: Place,
        label: &str,
        mark: PlaceMark,
        count: usize,
        cx: &mut Context<'_, SidebarView>,
    ) -> AnyElement {
        let theme = Theme::get(cx);
        let selected = place == current;
        let icon_color = if selected { theme.accent } else { theme.muted };
        let badge = match place {
            // Not organized only shows a count while the AI has work waiting.
            Place::NotOrganized if count == 0 => None,
            _ => Some(count),
        };
        let id = SharedString::from(format!("place-{label}"));
        row_frame(selected, theme)
            .id(id)
            .px(ROW_PADDING)
            .hover(|s| s.bg(theme.hover))
            .child(match mark {
                PlaceMark::Icon(name) => icon(name, px(15.), icon_color).into_any_element(),
                PlaceMark::Ai => ai_mark(px(15.), theme.ai).into_any_element(),
            })
            .child(div().flex_1().truncate().child(label.to_owned()))
            .when_some(badge.filter(|n| *n > 0), |s, n| {
                s.child(div().text_xs().text_color(theme.muted).child(n.to_string()))
            })
            .on_click(cx.listener(move |_, _, _, cx| cx.emit(SidebarEvent::Show(place))))
            .into_any_element()
    }
}

/// A sidebar row's shape, shared by places and folders: height, rounding, type and the
/// selected fill.
pub(super) fn row_frame(selected: bool, theme: &Theme) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .h(ROW_HEIGHT)
        .rounded(ROW_RADIUS)
        .text_sm()
        .cursor_pointer()
        .when(selected, |s| {
            s.bg(theme.hover).font_weight(FontWeight::MEDIUM)
        })
}
