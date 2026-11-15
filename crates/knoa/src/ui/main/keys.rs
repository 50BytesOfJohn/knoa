//! The main screen's actions and the keys bound to them.

use gpui::{App, KeyBinding, actions};

actions!(
    cards,
    [
        MoveLeft,
        MoveRight,
        MoveUp,
        MoveDown,
        OpenSelected,
        OpenSelectedInBackground,
        QuickLook,
        ToggleUpNext,
        CopyLink,
        DeleteSelected,
        Dismiss
    ]
);
actions!(
    viewer,
    [PreviousImage, NextImage, CloseViewer, CopyImage, SaveImage]
);
actions!(knoa, [FocusSearch, OpenSettings, NewLink]);

/// Bindings for the card grid and the image viewer. They live in their own key contexts so
/// they never fire while typing in the search field.
pub fn bind_keys(cx: &mut App) {
    let cards = Some("Cards");
    let viewer = Some("Viewer");
    cx.bind_keys([
        KeyBinding::new("left", MoveLeft, cards),
        KeyBinding::new("right", MoveRight, cards),
        KeyBinding::new("up", MoveUp, cards),
        KeyBinding::new("down", MoveDown, cards),
        KeyBinding::new("enter", OpenSelected, cards),
        KeyBinding::new("cmd-o", OpenSelected, cards),
        KeyBinding::new("cmd-down", OpenSelected, cards),
        KeyBinding::new("cmd-enter", OpenSelectedInBackground, cards),
        KeyBinding::new("space", QuickLook, cards),
        KeyBinding::new("l", ToggleUpNext, cards),
        KeyBinding::new("cmd-c", CopyLink, cards),
        KeyBinding::new("backspace", DeleteSelected, cards),
        KeyBinding::new("cmd-backspace", DeleteSelected, cards),
        KeyBinding::new("escape", Dismiss, cards),
        KeyBinding::new("escape", Dismiss, Some("Settings")),
        KeyBinding::new("left", PreviousImage, viewer),
        KeyBinding::new("right", NextImage, viewer),
        KeyBinding::new("space", CloseViewer, viewer),
        KeyBinding::new("escape", CloseViewer, viewer),
        KeyBinding::new("enter", OpenSelected, viewer),
        KeyBinding::new("cmd-o", OpenSelected, viewer),
        KeyBinding::new("cmd-c", CopyImage, viewer),
        KeyBinding::new("cmd-s", SaveImage, viewer),
        KeyBinding::new("cmd-f", FocusSearch, None),
        KeyBinding::new("cmd-k", FocusSearch, None),
        KeyBinding::new("cmd-,", OpenSettings, None),
        KeyBinding::new("cmd-n", NewLink, None),
    ]);
}
