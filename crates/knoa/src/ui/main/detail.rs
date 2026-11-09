//! The floating detail panel for the selected bookmark.

mod cover;
mod filing;
mod more;
mod sections;
mod tags;

pub(super) use tags::TagEdits;

use gpui::{
    Context, Div, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _, Stateful,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use knoa_browser::Activation;
use knoa_core::Bookmark;

use super::{CopyLink, DeleteSelected, MainView, ToggleUpNext};
use crate::ui::{
    components::{Tone, button, icon, kind_icon, site_mark, tip},
    theme::{DETAIL_WIDTH, MARGIN, PANEL_RADIUS, SERIF, Theme},
};

impl MainView {
    pub(super) fn render_detail(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let Some(bookmark) = self
            .selected
            .and_then(|id| self.library.read(cx).bookmark(id).cloned())
        else {
            return div().into_any_element();
        };
        let in_trash = bookmark.deleted_at.is_some();
        let body = div()
            .id("detail-body")
            .flex_1()
            .overflow_y_scroll()
            .child(self.detail_header(&bookmark, cx))
            .child(
                div()
                    .px_5()
                    .pt_5()
                    .pb_6()
                    .flex()
                    .flex_col()
                    .gap_6()
                    .children(self.detail_notice(&bookmark, cx))
                    .children(sections::fields(&bookmark, &theme))
                    .children(self.detail_tags(&bookmark, in_trash, cx))
                    // A bookmark in Trash is out of every folder's view, so it can't be filed.
                    .when(!in_trash, |s| s.child(self.detail_folders(&bookmark, cx)))
                    .children(sections::provenance(&bookmark, &theme)),
            );
        div()
            .absolute()
            .top(MARGIN)
            .right(MARGIN)
            .bottom(MARGIN)
            .w(DETAIL_WIDTH)
            .flex()
            .flex_col()
            .overflow_hidden()
            .rounded(PANEL_RADIUS)
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .shadow_lg()
            .child(body)
            .child(if in_trash {
                self.trash_actions(bookmark.id, cx)
            } else {
                self.detail_actions(&bookmark, cx)
            })
            .into_any_element()
    }

    /// Cover image, site and kind, title and summary: what the bookmark is, at a glance.
    fn detail_header(&self, bookmark: &Bookmark, cx: &mut Context<'_, MainView>) -> Div {
        let theme = Theme::get(cx).clone();
        let has_cover = bookmark.image.is_some();
        let mut header = div().relative().w_full().children(
            bookmark
                .image
                .as_ref()
                .map(|image| cover::image(image, &theme)),
        );
        let refresh = cover::refresh_status(bookmark.id, has_cover, cx);
        let (on_cover, under_byline) = if has_cover {
            (refresh, None)
        } else {
            (None, refresh)
        };
        header = header.child(
            div()
                .px_5()
                .pt(if has_cover { px(18.) } else { px(22.) })
                .flex()
                .flex_col()
                .gap_2()
                // Without an image, the close button sits on the byline's row.
                .child(self.byline(bookmark, cx).when(!has_cover, |s| s.pr(px(36.))))
                .children(under_byline.map(|pill| div().flex().child(pill)))
                .child(
                    div()
                        .font_family(SERIF)
                        .text_size(px(21.))
                        .line_height(px(27.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text)
                        .child(bookmark.title.clone()),
                )
                .children(sections::summary(bookmark, &theme)),
        );
        header
            .children(on_cover.map(|pill| div().absolute().top_3().left_3().child(pill)))
            .child(self.close_button(has_cover, cx))
    }

    fn byline(&self, bookmark: &Bookmark, cx: &mut Context<'_, MainView>) -> Div {
        let theme = Theme::get(cx);
        div()
            .flex()
            .items_center()
            .gap_1p5()
            .text_xs()
            .text_color(theme.muted)
            .child(site_mark(&bookmark.site, px(16.), cx))
            .child(div().min_w_0().truncate().child(bookmark.site.clone()))
            .when_some(bookmark.kind, |s, kind| {
                s.child(div().text_color(theme.faint).child("·"))
                    .child(icon(kind_icon(Some(kind)), px(12.), theme.muted))
                    .child(kind.label())
            })
    }

    /// On an image it takes the cards' dark badge look, so it reads on any picture.
    fn close_button(&self, on_image: bool, cx: &mut Context<'_, MainView>) -> Stateful<Div> {
        let theme = Theme::get(cx);
        let (bg, hover, fg) = if on_image {
            (
                gpui::black().opacity(0.45),
                gpui::black().opacity(0.7),
                gpui::white(),
            )
        } else {
            (gpui::transparent_black(), theme.soft, theme.muted)
        };
        div()
            .id("close-detail")
            .absolute()
            .top_3()
            .right_3()
            .size(px(28.))
            .rounded_full()
            .bg(bg)
            .hover(move |s| s.bg(hover))
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .child(icon("x", px(15.), fg))
            .on_click(cx.listener(|this, _, _, cx| {
                this.detail_open = false;
                cx.notify();
            }))
    }

    /// Open is the one primary action; the rest are icons with their shortcut in a tooltip.
    fn detail_actions(&self, bookmark: &Bookmark, cx: &mut Context<'_, MainView>) -> Div {
        let theme = Theme::get(cx).clone();
        let (url, id) = (bookmark.url.clone(), bookmark.id);
        let copy_url = url.clone();
        let copied = self
            .copied
            .as_ref()
            .is_some_and(|(copied, _)| *copied == id);
        let (queue_icon, queue_color, queue_tip) = if bookmark.up_next {
            ("bookmark-check", theme.accent, "Remove from Up next")
        } else {
            ("bookmark-plus", theme.muted, "Add to Up next")
        };
        div()
            .flex_none()
            .p_3()
            .flex()
            .items_center()
            .gap_1()
            .border_t_1()
            .border_color(theme.hairline)
            .child(
                button("open", "Open", Tone::Primary, cx)
                    .flex_1()
                    .mr_1()
                    .child(icon(
                        "corner-down-left",
                        px(13.),
                        theme.on_accent.opacity(0.6),
                    ))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.open_link(&url, Activation::Foreground, cx);
                    })),
            )
            .child(
                icon_button("up-next", queue_icon, queue_color, &theme)
                    .tooltip(tip(queue_tip, Some(&ToggleUpNext), &theme))
                    .on_click(cx.listener(move |this, _, _, cx| this.set_up_next(id, cx))),
            )
            .child(
                icon_button(
                    "copy",
                    if copied { "check" } else { "link" },
                    if copied { theme.accent } else { theme.muted },
                    &theme,
                )
                .tooltip(tip(
                    if copied { "Copied" } else { "Copy link" },
                    Some(&CopyLink),
                    &theme,
                ))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.copy_to_clipboard(id, copy_url.clone(), cx);
                })),
            )
            .child(
                icon_button("delete", "trash", theme.muted, &theme)
                    .tooltip(tip("Move to Trash", Some(&DeleteSelected), &theme))
                    .on_click(cx.listener(move |this, _, _, cx| this.move_to_trash(id, cx))),
            )
            .child(self.more_button(id, cx))
    }
}

fn icon_button(id: &'static str, name: &str, color: gpui::Hsla, theme: &Theme) -> Stateful<Div> {
    let hover = theme.soft;
    div()
        .id(id)
        .size(px(34.))
        .flex_none()
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .child(icon(name, px(16.), color))
}
