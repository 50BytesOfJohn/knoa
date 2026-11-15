//! How the viewer looks: a bar of actions, the picture as large as fits without
//! enlarging it, and a caption.

use gpui::{
    AnyElement, Context, Div, FontWeight, InteractiveElement as _, IntoElement, MouseButton,
    ObjectFit, ParentElement as _, Stateful, StatefulInteractiveElement as _, Styled as _,
    StyledImage as _, Window, div, img, prelude::FluentBuilder as _, px,
};
use knoa_core::{Bookmark, BookmarkId, Image};

use crate::ui::{
    components::{icon, tip_in},
    main::{CloseViewer, CopyImage, MainView, OpenSelected, SaveImage},
    theme::Theme,
};

const BAR_HEIGHT: f32 = 56.;
const CAPTION_HEIGHT: f32 = 84.;
/// Room on each side for the previous and next buttons.
const SIDE: f32 = 72.;
/// Clears the window's traffic lights, which stay on top of the viewer.
const TRAFFIC_LIGHTS: f32 = 104.;

impl MainView {
    pub(in crate::ui::main) fn render_viewer(
        &self,
        id: BookmarkId,
        notice: Option<&'static str>,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let bookmark = self.library.read(cx).bookmark(id).cloned();
        let pictures = self.viewable();
        let at = pictures.iter().position(|picture| *picture == id);
        let position = at.map(|at| format!("{} of {}", at + 1, pictures.len()));
        let (has_before, has_after) =
            at.map_or((false, false), |at| (at > 0, at + 1 < pictures.len()));
        let mut root = div()
            .id("viewer")
            .key_context("Viewer")
            .track_focus(&self.viewer_focus)
            .on_action(cx.listener(Self::close_viewer))
            .on_action(cx.listener(Self::previous_image))
            .on_action(cx.listener(Self::next_image))
            .on_action(cx.listener(Self::copy_image))
            .on_action(cx.listener(Self::save_image))
            .on_action(cx.listener(Self::open_selected))
            .absolute()
            .inset_0()
            .flex()
            .flex_col()
            .bg(gpui::black().opacity(0.95))
            .text_color(gpui::white())
            // Clicking the backdrop closes, as in a browser's lightbox.
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| this.close_overlay(window, cx)),
            )
            .child(self.viewer_bar(position, cx));
        // A bookmark deleted elsewhere while shown leaves the viewer empty but closable.
        if let Some(bookmark) = bookmark
            && let Some(image) = &bookmark.image
        {
            root = root
                .child(self.stage(image, (has_before, has_after), window, cx))
                .child(caption(&bookmark, notice));
        }
        root.into_any_element()
    }

    fn viewer_bar(&self, position: Option<String>, cx: &mut Context<'_, MainView>) -> Div {
        let theme = Theme::get(cx).clone();
        div()
            .flex_none()
            .h(px(BAR_HEIGHT))
            .pl(px(TRAFFIC_LIGHTS))
            .pr_4()
            .flex()
            .items_center()
            .gap_1()
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .text_color(gpui::white().opacity(0.6))
                    .children(position),
            )
            .child(
                bar_button("viewer-open", "external-link")
                    .tooltip(tip_in(
                        "Open original",
                        Some(&OpenSelected),
                        "Viewer",
                        &theme,
                    ))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.open_selected(&OpenSelected, window, cx);
                    })),
            )
            .child(
                bar_button("viewer-copy", "copy")
                    .tooltip(tip_in("Copy image", Some(&CopyImage), "Viewer", &theme))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.copy_image(&CopyImage, window, cx);
                    })),
            )
            .child(
                bar_button("viewer-save", "download")
                    .tooltip(tip_in("Save image…", Some(&SaveImage), "Viewer", &theme))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.save_image(&SaveImage, window, cx);
                    })),
            )
            .child(
                bar_button("viewer-close", "x")
                    .tooltip(tip_in("Close", Some(&CloseViewer), "Viewer", &theme))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.close_viewer(&CloseViewer, window, cx);
                    })),
            )
    }

    /// The picture, contain-fit in the space between the bar and the caption, and never
    /// enlarged past its stored size, so a small image isn't blown up into a blur.
    fn stage(
        &self,
        image: &Image,
        (has_before, has_after): (bool, bool),
        window: &Window,
        cx: &mut Context<'_, MainView>,
    ) -> Div {
        let viewport = window.viewport_size();
        let room_width = f32::from(viewport.width) - SIDE * 2.;
        let room_height = f32::from(viewport.height) - BAR_HEIGHT - CAPTION_HEIGHT;
        let (width, height) = (image.width.max(1) as f32, image.height.max(1) as f32);
        let scale = (room_width / width).min(room_height / height).min(1.);
        let picture = img(image.path.clone())
            .w(px(width * scale))
            .h(px(height * scale))
            .rounded(px(4.))
            .object_fit(ObjectFit::Contain);
        div()
            .flex_1()
            .min_h_0()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .id("viewer-picture")
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(picture),
            )
            .when(has_before, |s| {
                s.child(
                    side_button("viewer-previous", "chevron-left")
                        .left_4()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.step_viewer(false, window, cx);
                        })),
                )
            })
            .when(has_after, |s| {
                s.child(
                    side_button("viewer-next", "chevron-right")
                        .right_4()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.step_viewer(true, window, cx);
                        })),
                )
            })
    }
}

/// Title, then site, resolution and format: what the picture is and where it's from.
fn caption(bookmark: &Bookmark, notice: Option<&'static str>) -> Div {
    let field = |name: &str| {
        bookmark
            .fields
            .get(name)
            .and_then(|value| value.as_str())
            .map(str::to_owned)
    };
    let details: Vec<String> = [
        Some(bookmark.site.clone()),
        field("resolution"),
        field("format"),
    ]
    .into_iter()
    .flatten()
    .filter(|text| !text.is_empty())
    .collect();
    div()
        .flex_none()
        .h(px(CAPTION_HEIGHT))
        .px(px(SIDE))
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .child(
            div()
                .max_w_full()
                .truncate()
                .text_base()
                .font_weight(FontWeight::MEDIUM)
                .child(bookmark.title.clone()),
        )
        .child(
            div()
                .max_w_full()
                .truncate()
                .text_sm()
                .text_color(gpui::white().opacity(0.6))
                .child(details.join(" · ")),
        )
        .children(notice.map(|text| {
            div()
                .mt_1()
                .px_2p5()
                .h(px(22.))
                .flex()
                .items_center()
                .rounded_full()
                .bg(gpui::white().opacity(0.14))
                .text_xs()
                .child(text)
        }))
}

fn bar_button(id: &'static str, glyph: &str) -> Stateful<Div> {
    div()
        .id(id)
        .size(px(34.))
        .flex_none()
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|s| s.bg(gpui::white().opacity(0.12)))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(icon(glyph, px(17.), gpui::white().opacity(0.85)))
}

fn side_button(id: &'static str, glyph: &str) -> Stateful<Div> {
    div()
        .id(id)
        .absolute()
        .top(gpui::relative(0.5))
        .mt(px(-20.))
        .size(px(40.))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .bg(gpui::white().opacity(0.08))
        .hover(|s| s.bg(gpui::white().opacity(0.18)))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(icon(glyph, px(20.), gpui::white()))
}
