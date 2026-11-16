//! The main screen's actions and the keys bound to them.

use gpui::{Action, App, KeyBinding, actions};

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
        ShowDetails,
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

/// The keys for the first nine folders in the sidebar, which shows them beside each.
const FOLDER_KEYS: [&str; 9] = [
    "cmd-1", "cmd-2", "cmd-3", "cmd-4", "cmd-5", "cmd-6", "cmd-7", "cmd-8", "cmd-9",
];

/// Shows the sidebar's `n`th visible folder, counting from 0.
#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = knoa, no_json)]
pub struct SelectFolder(pub usize);

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
        KeyBinding::new("cmd-i", ShowDetails, cards),
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
        KeyBinding::new("cmd-i", ShowDetails, viewer),
        KeyBinding::new("cmd-f", FocusSearch, None),
        KeyBinding::new("cmd-k", FocusSearch, None),
        KeyBinding::new("cmd-,", OpenSettings, None),
        KeyBinding::new("cmd-n", NewLink, None),
    ]);
    let folders = FOLDER_KEYS
        .iter()
        .enumerate()
        .map(|(n, key)| KeyBinding::new(key, SelectFolder(n), Some("Main")));
    cx.bind_keys(folders);
}
