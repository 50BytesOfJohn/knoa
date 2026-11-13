//! The workspace page's tag list: tags the AI should use, each with a short optional
//! hint, and whether the AI may add tags of its own.

use gpui::{
    AnyElement, App, AppContext as _, Context, Div, Entity, EventEmitter, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _,
    Subscription, Window, div, px,
};
use gpui_component::{
    Disableable as _, Sizable as _,
    input::{Input, InputEvent, InputState},
    switch::Switch,
};
use knoa_core::{
    ListedTag, MAX_LISTED_TAGS, MAX_TAG_DESCRIPTION_CHARS, TagList, normalize_tag, tag_fits,
};

use super::form;
use crate::ui::{
    components::{icon, tip},
    theme::Theme,
};

/// The list was edited and should be saved.
pub struct TagListChanged;

impl EventEmitter<TagListChanged> for TagListEditor {}

pub struct TagListEditor {
    rows: Vec<TagRow>,
    only_these: bool,
    new_tag: Entity<InputState>,
    _new_tag_events: Subscription,
}

struct TagRow {
    name: String,
    description: Entity<InputState>,
    _events: Subscription,
}

impl TagListEditor {
    pub fn new(list: &TagList, window: &mut Window, cx: &mut Context<'_, TagListEditor>) -> Self {
        let new_tag = cx.new(|cx| InputState::new(window, cx).placeholder("Add a tag"));
        let new_tag_events = cx.subscribe_in(&new_tag, window, Self::on_new_tag_event);
        let mut editor = TagListEditor {
            rows: Vec::new(),
            only_these: list.only_these,
            new_tag,
            _new_tag_events: new_tag_events,
        };
        for tag in &list.tags {
            editor.insert_row(tag, window, cx);
        }
        editor
    }

    pub fn tag_list(&self, cx: &App) -> TagList {
        TagList {
            tags: self
                .rows
                .iter()
                .map(|row| ListedTag {
                    name: row.name.clone(),
                    description: row.description.read(cx).value().trim().to_owned(),
                })
                .collect(),
            only_these: self.only_these,
        }
    }

    pub(super) fn refresh(
        &mut self,
        list: &TagList,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        if self.tag_list(cx).tags != list.tags {
            self.rows.clear();
            for tag in &list.tags {
                self.insert_row(tag, window, cx);
            }
        }
        self.only_these = list.only_these;
        cx.notify();
    }

    fn is_full(&self) -> bool {
        self.rows.len() >= MAX_LISTED_TAGS
    }

    /// Adds every comma-separated tag in `text` that isn't listed yet, while there's room.
    fn add(&mut self, text: &str, window: &mut Window, cx: &mut Context<'_, TagListEditor>) {
        let mut added = false;
        for name in text.split(',').map(normalize_tag) {
            if !tag_fits(&name) || self.is_full() || self.rows.iter().any(|r| r.name == name) {
                continue;
            }
            let tag = ListedTag {
                name,
                description: String::new(),
            };
            self.insert_row(&tag, window, cx);
            added = true;
        }
        if added {
            cx.emit(TagListChanged);
            cx.notify();
        }
    }

    /// Keeps rows sorted by name, as the store returns them.
    fn insert_row(
        &mut self,
        tag: &ListedTag,
        window: &mut Window,
        cx: &mut Context<'_, TagListEditor>,
    ) {
        let description = cx.new(|cx| {
            let mut input = InputState::new(window, cx).placeholder("When to use it (optional)");
            input.set_value(tag.description.clone(), window, cx);
            input
        });
        let events = cx.subscribe_in(&description, window, Self::on_description_event);
        let at = self.rows.partition_point(|row| row.name < tag.name);
        self.rows.insert(
            at,
            TagRow {
                name: tag.name.clone(),
                description,
                _events: events,
            },
        );
    }

    /// Emptying the list turns the switch off too, so it never shows on while disabled.
    fn remove(&mut self, name: &str, cx: &mut Context<'_, TagListEditor>) {
        self.rows.retain(|row| row.name != name);
        if self.rows.is_empty() {
            self.only_these = false;
        }
        cx.emit(TagListChanged);
        cx.notify();
    }

    /// Enter or a comma adds what's typed and keeps the field open for the next tag.
    fn on_new_tag_event(
        &mut self,
        input: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<'_, TagListEditor>,
    ) {
        let text = input.read(cx).value();
        match event {
            InputEvent::PressEnter { .. } => {
                self.add(&text, window, cx);
                input.update(cx, |input, cx| input.set_value("", window, cx));
            }
            InputEvent::Change => {
                if let Some((done, rest)) = text.rsplit_once(',') {
                    self.add(done, window, cx);
                    let rest = rest.trim_start().to_owned();
                    input.update(cx, |input, cx| input.set_value(rest, window, cx));
                }
            }
            InputEvent::Blur | InputEvent::Focus => {}
        }
    }

    /// A hint goes into every organizing request, so it's cut at its limit as it's typed.
    fn on_description_event(
        &mut self,
        input: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<'_, TagListEditor>,
    ) {
        match event {
            InputEvent::Change => {
                let text = input.read(cx).value();
                if text.chars().count() > MAX_TAG_DESCRIPTION_CHARS {
                    let cut: String = text.chars().take(MAX_TAG_DESCRIPTION_CHARS).collect();
                    input.update(cx, |input, cx| input.set_value(cut, window, cx));
                }
            }
            InputEvent::Blur | InputEvent::PressEnter { .. } => cx.emit(TagListChanged),
            InputEvent::Focus => {}
        }
    }

    fn list(&self, theme: &Theme, cx: &mut Context<'_, TagListEditor>) -> Div {
        let mut list = div().flex().flex_col().gap_1();
        for (index, row) in self.rows.iter().enumerate() {
            list = list.child(self.tag_row(index, row, theme, cx));
        }
        let count = format!("{} of {MAX_LISTED_TAGS}", self.rows.len());
        let field = div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                div().flex_1().child(
                    Input::new(&self.new_tag)
                        .small()
                        .disabled(self.is_full())
                        .prefix(div().text_color(theme.faint).child("#")),
                ),
            )
            .child(
                div()
                    .w(px(64.))
                    .flex()
                    .justify_end()
                    .text_xs()
                    .text_color(if self.is_full() {
                        theme.danger
                    } else {
                        theme.faint
                    })
                    .child(count),
            );
        list.child(div().pt_1().child(field))
    }

    fn tag_row(
        &self,
        index: usize,
        row: &TagRow,
        theme: &Theme,
        cx: &mut Context<'_, TagListEditor>,
    ) -> AnyElement {
        let name = row.name.clone();
        let (hover, faint) = (theme.soft, theme.faint);
        let remove = div()
            .id(("unlist-tag", index))
            .size(px(24.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(6.))
            .cursor_pointer()
            .hover(move |s| s.bg(hover))
            .child(icon("x", px(13.), faint))
            .tooltip(tip("Remove from list", None, theme))
            .on_click(cx.listener(move |this, _, _, cx| this.remove(&name, cx)));
        div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .w(px(150.))
                    .flex_none()
                    .flex()
                    .gap_1()
                    .text_sm()
                    .overflow_hidden()
                    .child(div().text_color(theme.faint).child("#"))
                    .child(div().truncate().child(row.name.clone())),
            )
            .child(div().flex_1().child(Input::new(&row.description).small()))
            .child(div().w(px(64.)).flex().justify_end().child(remove))
            .into_any_element()
    }
}

impl Render for TagListEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let mut rows = vec![form::stacked(
            "Tags the AI should use",
            Some(
                "When one of these fits a bookmark, the AI uses it instead of a word of its own. A short hint tells it when one fits. You can also right-click a tag on a bookmark to list it.",
            ),
            self.list(&theme, cx),
            &theme,
        )];
        let empty = self.rows.is_empty();
        // Shown even with nothing listed, so the choice is known before the first tag.
        let choice = "Off: the AI adds its own tags for what the list doesn't cover. On: bookmarks get only listed tags, or none when nothing fits.";
        let about = if empty {
            format!("Add a tag first. {choice}")
        } else {
            choice.to_owned()
        };
        rows.push(form::row(
            "Only use tags from this list",
            Some(&about),
            Switch::new("only-listed-tags")
                .checked(self.only_these)
                .disabled(empty)
                .on_click(cx.listener(|this, on: &bool, _, cx| {
                    this.only_these = *on;
                    cx.emit(TagListChanged);
                    cx.notify();
                })),
            &theme,
        ));
        if !empty {
            rows.push(form::note(
                "Bookmarks organized from now on get these tags. To bring them to the ones already organized, organize again.",
                theme.muted,
            ));
        }
        form::group("Tag list", rows, &theme)
    }
}
