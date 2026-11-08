//! What changes while tags are edited: the tags just taken off, and the field a new one
//! is typed in.

use gpui::{AppContext as _, Context, Entity, Focusable as _, Subscription, Window};
use gpui_component::input::{InputEvent, InputState};
use knoa_core::{BookmarkId, Origin, normalize_tag};

use crate::ui::main::MainView;

/// Tag changes on the bookmark in the detail panel. It lives as long as that bookmark
/// stays selected.
pub(in crate::ui::main) struct TagEdits {
    pub(super) bookmark: BookmarkId,
    /// Tags taken off here, kept in place and faded so a slip is one click to undo.
    pub(super) removed: Vec<(String, Origin)>,
    pub(super) field: Option<TagField>,
}

/// The field a new tag is typed in, while it's open.
pub(super) struct TagField {
    pub(super) input: Entity<InputState>,
    _events: Subscription,
}

impl TagEdits {
    pub(in crate::ui::main) fn is_typing(&self, window: &Window, cx: &gpui::App) -> bool {
        self.field
            .as_ref()
            .is_some_and(|field| field.input.read(cx).focus_handle(cx).is_focused(window))
    }
}

impl MainView {
    /// Removed tags can be put back only while their bookmark stays selected.
    pub(in crate::ui::main) fn forget_stale_tag_edits(&mut self) {
        if self
            .tag_edits
            .as_ref()
            .is_some_and(|edits| Some(edits.bookmark) != self.selected)
        {
            self.tag_edits = None;
        }
    }

    pub(super) fn tag_field(&self, bookmark: BookmarkId) -> Option<&TagField> {
        self.tag_edits
            .as_ref()
            .filter(|edits| edits.bookmark == bookmark)
            .and_then(|edits| edits.field.as_ref())
    }

    fn edits_for(&mut self, bookmark: BookmarkId) -> &mut TagEdits {
        if !matches!(&self.tag_edits, Some(edits) if edits.bookmark == bookmark) {
            self.tag_edits = None;
        }
        self.tag_edits.get_or_insert_with(|| TagEdits {
            bookmark,
            removed: Vec::new(),
            field: None,
        })
    }

    pub(super) fn remove_tag(
        &mut self,
        bookmark: BookmarkId,
        name: String,
        origin: Origin,
        cx: &mut Context<'_, MainView>,
    ) {
        self.edits_for(bookmark)
            .removed
            .push((name.clone(), origin));
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.remove_tag(bookmark, &name));
        });
        cx.notify();
    }

    /// Puts the tag back as whoever added it, so an AI tag stays marked as the AI's.
    pub(super) fn put_back_tag(
        &mut self,
        bookmark: BookmarkId,
        name: String,
        origin: Origin,
        cx: &mut Context<'_, MainView>,
    ) {
        self.edits_for(bookmark).removed.retain(|(r, _)| *r != name);
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.add_tag(bookmark, &name, origin));
        });
        cx.notify();
    }

    /// Adds every comma-separated tag in `text`.
    pub(super) fn add_tags(
        &mut self,
        bookmark: BookmarkId,
        text: &str,
        cx: &mut Context<'_, MainView>,
    ) {
        let names: Vec<String> = text
            .split(',')
            .map(normalize_tag)
            .filter(|name| !name.is_empty())
            .collect();
        if names.is_empty() {
            return;
        }
        let edits = self.edits_for(bookmark);
        edits.removed.retain(|(r, _)| !names.contains(r));
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| {
                names
                    .iter()
                    .try_for_each(|name| store.add_tag(bookmark, name, Origin::User))
            });
        });
        cx.notify();
    }

    pub(super) fn open_tag_field(
        &mut self,
        bookmark: BookmarkId,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Add a tag"));
        input.update(cx, |input, cx| input.focus(window, cx));
        let events = cx.subscribe_in(&input, window, move |this, input, event, window, cx| {
            this.on_tag_field_event(bookmark, input, event, window, cx);
        });
        self.edits_for(bookmark).field = Some(TagField {
            input,
            _events: events,
        });
        cx.notify();
    }

    /// Enter or a comma adds what's typed and stays open for the next tag; Enter on an
    /// empty field, or leaving it empty, closes it.
    fn on_tag_field_event(
        &mut self,
        bookmark: BookmarkId,
        input: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let text = input.read(cx).value();
        match event {
            InputEvent::PressEnter { .. } if text.trim().is_empty() => {
                self.close_tag_field(window, cx);
            }
            InputEvent::PressEnter { .. } => {
                self.add_tags(bookmark, &text, cx);
                self.clear_tag_field(window, cx);
            }
            InputEvent::Blur if text.trim().is_empty() => {
                if let Some(edits) = &mut self.tag_edits {
                    edits.field = None;
                }
                cx.notify();
            }
            // A comma ends a tag, as in most tag fields; what follows it stays to type on.
            InputEvent::Change if text.contains(',') => {
                if let Some((done, rest)) = text.rsplit_once(',') {
                    self.add_tags(bookmark, done, cx);
                    let rest = rest.trim_start().to_owned();
                    input.update(cx, |input, cx| input.set_value(rest, window, cx));
                }
            }
            // Suggestions follow the text.
            InputEvent::Change => cx.notify(),
            _ => {}
        }
    }

    pub(super) fn clear_tag_field(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        if let Some(field) = self.tag_edits.as_ref().and_then(|e| e.field.as_ref()) {
            field.input.update(cx, |input, cx| {
                input.set_value("", window, cx);
                input.focus(window, cx);
            });
        }
    }

    pub(super) fn close_tag_field(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        if let Some(edits) = &mut self.tag_edits {
            edits.field = None;
        }
        self.cards_focus.focus(window, cx);
        cx.notify();
    }
}
