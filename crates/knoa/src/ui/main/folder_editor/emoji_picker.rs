//! The folder editor's emoji: a few that suit common folders, and a field that takes any
//! one emoji, typed or from the system's emoji picker.

use gpui::{
    AppContext as _, Context, Div, Entity, InteractiveElement as _, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, Subscription, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{
    Sizable as _,
    input::{Input, InputEvent, InputState},
};
use knoa_core::{Emoji, FolderIcon};

use super::FolderEditor;
use crate::{
    i18n::t,
    ui::theme::{ROW_RADIUS, Theme},
};

/// Emoji for the folders people make most: reading, work, media, food, travel, home.
const SUGGESTED: [&str; 18] = [
    "📚", "📰", "💻", "💼", "🎨", "🎵", "🎬", "📷", "🎮", "🍳", "✈️", "🏠", "🛒", "💡", "🌱", "❤️",
    "⭐", "📌",
];
const CELL: f32 = 30.;
const EMOJI_SIZE: f32 = 17.;
const FIELD_WIDTH: f32 = 112.;

/// The field, holding the folder's emoji when it has one.
pub(super) fn emoji_state(
    icon: Option<&FolderIcon>,
    window: &mut Window,
    cx: &mut Context<'_, FolderEditor>,
) -> Entity<InputState> {
    let current = emoji_of(icon).unwrap_or_default();
    cx.new(|cx| {
        let mut state =
            InputState::new(window, cx).placeholder(t!("main-folder-emoji-placeholder"));
        state.set_value(current, window, cx);
        state
    })
}

/// An emoji typed or picked into the field becomes the icon. Anything else typed is
/// dropped, keeping the last emoji.
pub(super) fn follow_emoji(
    field: &Entity<InputState>,
    window: &mut Window,
    cx: &mut Context<'_, FolderEditor>,
) -> Subscription {
    cx.subscribe_in(
        field,
        window,
        |this, field, event: &InputEvent, window, cx| {
            if !matches!(event, InputEvent::Change) {
                return;
            }
            let text = field.read(cx).value().to_string();
            let Some(emoji) = last_emoji(&text) else {
                return;
            };
            if emoji.as_str() != text {
                let kept = emoji.as_str().to_owned();
                field.update(cx, |field, cx| field.set_value(kept, window, cx));
            }
            this.icon = Some(FolderIcon::Emoji(emoji));
            cx.notify();
        },
    )
}

/// The last whole emoji in `text`. The longest suffix that is one emoji wins, so a
/// sequence such as 👩🏽‍💻 isn't cut down to its last part.
fn last_emoji(text: &str) -> Option<Emoji> {
    let text = text.trim();
    text.char_indices()
        .find_map(|(at, _)| Emoji::new(&text[at..]))
}

fn emoji_of(icon: Option<&FolderIcon>) -> Option<String> {
    match icon {
        Some(FolderIcon::Emoji(emoji)) => Some(emoji.as_str().to_owned()),
        _ => None,
    }
}

impl FolderEditor {
    pub(super) fn emoji_picker(&self, theme: &Theme, cx: &mut Context<'_, FolderEditor>) -> Div {
        let mut grid = div().flex().flex_wrap().gap_1();
        for (index, emoji) in SUGGESTED.iter().enumerate() {
            grid = grid.child(self.emoji_cell(index, emoji, theme, cx));
        }
        let typed = emoji_of(self.icon.as_ref())
            .is_some_and(|current| !SUGGESTED.contains(&current.as_str()));
        let field_color = theme.folder_color(self.color);
        div().flex().flex_col().gap_2().child(grid).child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .text_sm()
                .text_color(theme.muted)
                .child(t!("main-folder-any-emoji"))
                .child(
                    div().w(px(FIELD_WIDTH)).child(
                        Input::new(&self.emoji)
                            .small()
                            .rounded(ROW_RADIUS)
                            .when(typed, |s| s.bg(field_color.opacity(0.16))),
                    ),
                ),
        )
    }

    fn emoji_cell(
        &self,
        index: usize,
        emoji: &'static str,
        theme: &Theme,
        cx: &mut Context<'_, FolderEditor>,
    ) -> impl gpui::IntoElement {
        let selected = matches!(&self.icon, Some(FolderIcon::Emoji(e)) if e.as_str() == emoji);
        let color = theme.folder_color(self.color);
        let hover = theme.hover;
        div()
            .id(("folder-emoji", index))
            .size(px(CELL))
            .flex()
            .items_center()
            .justify_center()
            .rounded(ROW_RADIUS)
            .cursor_pointer()
            .text_size(px(EMOJI_SIZE))
            .when(selected, |s| s.bg(color.opacity(0.16)))
            .when(!selected, |s| s.hover(move |s| s.bg(hover)))
            .child(SharedString::from(emoji))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.icon = Emoji::new(emoji).map(FolderIcon::Emoji);
                this.emoji
                    .update(cx, |field, cx| field.set_value("", window, cx));
                cx.notify();
            }))
    }

    /// Clears the field when a glyph or the plain shape replaces an emoji.
    pub(super) fn clear_typed_emoji(&self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.emoji
            .update(cx, |field, cx| field.set_value("", window, cx));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_field_keeps_the_last_whole_emoji_typed() {
        let kept = |text| last_emoji(text).map(|e| e.as_str().to_owned());
        assert_eq!(kept("🎨").as_deref(), Some("🎨"));
        assert_eq!(kept("🎨📚").as_deref(), Some("📚"));
        assert_eq!(kept("a👩🏽‍💻").as_deref(), Some("👩🏽‍💻"));
        assert_eq!(kept("abc"), None);
    }
}
