//! The suggestion list under the pill while the user types: drawn over the cards without
//! taking focus from the input, which keeps the keys.

use gpui::{
    AnyElement, Context, Div, Focusable as _, FontWeight, HighlightStyle, InteractiveElement as _,
    IntoElement, MouseButton, ObjectFit, ParentElement as _, StatefulInteractiveElement as _,
    Styled as _, StyledImage as _, StyledText, Window, deferred, div, img,
    prelude::FluentBuilder as _, px,
};
use knoa_browser::Activation;
use knoa_core::{Filter, Place};

use super::{
    MainView, PILL_HEIGHT,
    suggest::{Action, Group, Row, suggestions},
};
use crate::{
    i18n::t,
    state::search::Token,
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
    pub(super) fn suggestion_rows(&self, window: &Window, cx: &Context<'_, MainView>) -> Vec<Row> {
        let focused = self.search.read(cx).focus_handle(cx).is_focused(window);
        let library = self.library.read(cx);
        let search = library.search();
        if !focused || self.overlay.is_some() || search.typed.trim().is_empty() {
            return Vec::new();
        }
        suggestions(
            &search.typed,
            search,
            library.place(),
            library.base_facets(),
            &library.results(),
            library.folder_tree(),
        )
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

    /// ↑ and ↓ move the highlight while the list is open. Returns whether it was.
    pub(super) fn move_highlight(
        &mut self,
        down: bool,
        window: &Window,
        cx: &mut Context<'_, MainView>,
    ) -> bool {
        let count = self.suggestion_rows(window, cx).len();
        if count == 0 {
            return false;
        }
        let at = self.pill.highlight.min(count - 1);
        self.pill.highlight = if down {
            (at + 1) % count
        } else {
            (at + count - 1) % count
        };
        cx.notify();
        true
    }

    /// ↵: the highlighted row. Returns whether the list was open.
    pub(super) fn apply_highlighted(
        &mut self,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> bool {
        if self.suggestion_rows(window, cx).is_empty() {
            return false;
        }
        self.apply_suggestion(self.pill.highlight, window, cx);
        true
    }

    /// ⇥: the first kind, folder, tag, site or field row. Returns whether there was one.
    pub(super) fn apply_first_filter(
        &mut self,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> bool {
        let rows = self.suggestion_rows(window, cx);
        let Some(index) = rows.iter().position(Row::is_filter) else {
            return false;
        };
        self.apply_suggestion(index, window, cx);
        true
    }

    fn apply_suggestion(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let Some(row) = self.suggestion_rows(window, cx).into_iter().nth(index) else {
            return;
        };
        let mut search = self.library.read(cx).search().clone();
        match row.action {
            Action::SearchFor => search.lock_in_typed(),
            Action::Add(filter) => {
                search.typed.clear();
                search.add(Token::Filter(filter));
            }
            Action::Folder(id) => {
                search.typed.clear();
                self.show_place(Place::Folder(id), cx);
            }
            Action::Open(id) => {
                self.open_bookmark(id, Activation::Foreground, cx);
                return;
            }
        }
        self.set_search(search, window, cx);
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
