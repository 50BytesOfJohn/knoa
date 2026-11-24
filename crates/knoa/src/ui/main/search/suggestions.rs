//! The suggestion list under the pill while the user types: drawn over the cards without
//! taking focus from the input, which keeps the keys.

use std::rc::Rc;

use gpui::{
    AnyElement, Context, Div, Focusable as _, FontWeight, HighlightStyle, InteractiveElement as _,
    IntoElement, MouseButton, ObjectFit, ParentElement as _, StatefulInteractiveElement as _,
    Styled as _, StyledImage as _, StyledText, Window, deferred, div, img,
    prelude::FluentBuilder as _, px,
};
use knoa_core::Filter;

use super::{
    MainView, PILL_HEIGHT,
    suggest::{Action, Group, Row, suggestions},
};
use crate::{
    i18n::t,
    ui::{
        components::{Finish, folder_mark, icon, kind_icon, site_mark, tabular_figures},
        theme::Theme,
    },
};

const ROW_HEIGHT: f32 = 32.;
const GAP_BELOW_PILL: f32 = 6.;

impl MainView {
    /// The rows for what is typed now, or none while the list is closed: it is open while
    /// the pill has focus and holds typed text.
    pub(super) fn suggestion_rows(
        &mut self,
        window: &Window,
        cx: &Context<'_, MainView>,
    ) -> Rc<[Row]> {
        let focused = self.search.read(cx).focus_handle(cx).is_focused(window);
        let library = self.library.read(cx);
        let search = library.search();
        if !focused || self.overlay.is_some() || search.typed.trim().is_empty() {
            return Rc::default();
        }
        let place = library.place();
        let build = || {
            let results = library.results();
            let rows = suggestions(
                &search.typed,
                search,
                place,
                library.base_facets(),
                &results,
                library.folder_tree(),
            );
            rows.into()
        };
        self.pill.suggestion_rows.get(search, place, build).clone()
    }

    /// The list `width` wide under the pill, deferred so it floats over the cards.
    pub(super) fn render_suggestions(
        &mut self,
        width: f32,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> Option<AnyElement> {
        let rows = self.suggestion_rows(window, cx);
        if rows.is_empty() {
            return None;
        }
        let theme = Theme::get(cx).clone();
        self.pill.highlight = self.pill.highlight.min(rows.len() - 1);
        let mut list = div()
            .id("suggestions")
            .occlude()
            .absolute()
            .top(px(PILL_HEIGHT + GAP_BELOW_PILL))
            .left_0()
            .w(px(width))
            .p(px(6.))
            .flex()
            .flex_col()
            .rounded(px(16.))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .shadow_lg()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());
        let mut group = None;
        for (index, row) in rows.iter().enumerate() {
            if row.group != group {
                group = row.group;
                if let Some(group) = group {
                    list = list.child(heading(group, &theme));
                }
            }
            list = list.child(self.search_suggestion(index, row, &theme, cx));
        }
        Some(deferred(list.child(footer(&theme))).into_any_element())
    }

    fn search_suggestion(
        &self,
        index: usize,
        row: &Row,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let highlighted = index == self.pill.highlight;
        let mut label = StyledText::new(row.label.clone());
        if !matches!(row.action, Action::SearchFor | Action::Open(_)) {
            let completion = HighlightStyle {
                font_weight: Some(FontWeight::SEMIBOLD),
                ..HighlightStyle::default()
            };
            let ranges = [0..row.typed.start, row.typed.end..row.label.len()];
            label = label.with_highlights(
                ranges
                    .into_iter()
                    .filter(|r| !r.is_empty())
                    .map(|r| (r, completion)),
            );
        }
        div()
            .id(("suggestion", index))
            .flex_none()
            .h(px(ROW_HEIGHT))
            .px(px(10.))
            .flex()
            .items_center()
            .gap(px(10.))
            .rounded(px(9.))
            .text_sm()
            .cursor_pointer()
            .when(highlighted, |s| s.bg(theme.hover))
            .child(self.row_mark(row, theme, cx))
            .child(div().flex_1().min_w_0().truncate().child(label))
            .when_some(self.row_detail(row, cx), |s, detail| {
                s.child(
                    div()
                        .flex_none()
                        .text_xs()
                        .text_color(theme.faint)
                        .child(detail),
                )
            })
            .when_some(row.count, |s, count| {
                s.child(
                    div()
                        .flex_none()
                        .text_size(px(12.))
                        .font_features(tabular_figures())
                        .text_color(theme.faint)
                        .child(crate::format::number(count as f64)),
                )
            })
            .when(highlighted, |s| {
                s.child(icon("corner-down-left", px(12.), theme.faint))
            })
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                if *hovered && this.pill.highlight != index {
                    this.pill.highlight = index;
                    cx.notify();
                }
            }))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.apply_suggestion(index, window, cx);
            }))
            .into_any_element()
    }

    /// A bookmark row's site, after its title.
    fn row_detail(&self, row: &Row, cx: &Context<'_, MainView>) -> Option<String> {
        let Action::Open(id) = row.action else {
            return None;
        };
        self.library.read(cx).bookmark(id).map(|b| b.site.clone())
    }

    fn row_mark(&self, row: &Row, theme: &Theme, cx: &Context<'_, MainView>) -> AnyElement {
        let size = px(14.);
        let name = match &row.action {
            Action::SearchFor => "search",
            Action::Add(Filter::Kind(kind)) => kind_icon(Some(*kind)),
            Action::Add(Filter::Tag(_)) => "hash",
            Action::Add(Filter::Site(_)) => "globe",
            Action::Add(Filter::Field(_)) => "text-align-start",
            Action::Folder(id) => {
                let folder = self.library.read(cx).folder_tree().get(*id);
                let (glyph, color) = folder.map_or((None, theme.muted), |f| {
                    (f.icon.as_ref(), theme.folder_color(f.color))
                });
                return folder_mark(glyph, color, size, Finish::Line).into_any_element();
            }
            Action::Open(id) => return self.thumbnail(*id, cx),
        };
        icon(name, size, theme.muted).into_any_element()
    }

    /// 30 × 24, the bookmark's picture or its site's mark.
    fn thumbnail(&self, id: knoa_core::BookmarkId, cx: &Context<'_, MainView>) -> AnyElement {
        let bookmark = self.library.read(cx).bookmark(id);
        let frame = div()
            .flex_none()
            .w(px(30.))
            .h(px(24.))
            .flex()
            .items_center()
            .justify_center();
        match bookmark {
            Some(bookmark) => match &bookmark.image {
                Some(image) => frame
                    .child(
                        img(image.path.clone())
                            .size_full()
                            .rounded(px(5.))
                            .object_fit(ObjectFit::Cover),
                    )
                    .into_any_element(),
                None => frame
                    .child(site_mark(&bookmark.site, px(18.), cx))
                    .into_any_element(),
            },
            None => frame.into_any_element(),
        }
    }
}

fn heading(group: Group, theme: &Theme) -> Div {
    div()
        .flex_none()
        .pt(px(8.))
        .pb(px(2.))
        .px(px(10.))
        .text_size(px(11.))
        .text_color(theme.faint)
        .child(group.heading().to_uppercase())
}

fn footer(theme: &Theme) -> Div {
    div()
        .flex_none()
        .mt(px(6.))
        .pt(px(6.))
        .px(px(10.))
        .border_t_1()
        .border_color(theme.hairline)
        .text_size(px(11.5))
        .text_color(theme.faint)
        .child(t!("main-suggest-footer"))
}
