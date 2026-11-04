//! The window's root view.

use gpui::{Context, IntoElement, ParentElement as _, Render, Styled as _, Window, div, rgb};

use crate::app::APP_NAME;

/// For now the window only shows the app's name; the library UI lands here next.
pub struct RootView;

impl RootView {
    pub fn new() -> Self {
        Self
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(0x00fa_fafa))
            .text_color(rgb(0x0018_1818))
            .text_3xl()
            .child(APP_NAME)
    }
}
