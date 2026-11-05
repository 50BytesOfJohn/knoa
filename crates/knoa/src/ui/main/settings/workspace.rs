//! A workspace's page: name and color, the browsers it syncs and opens links in, how it's
//! organized, and an optional guideline for what it keeps. The same page creates one.
//!
//! An existing workspace saves as you go, like the rest of settings: switches and colors
//! at once, text when its field loses focus or the page is left.

mod browsers;

use gpui::{
    AppContext as _, Context, Div, Entity, EventEmitter, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _, Subscription,
    TaskExt as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    input::{Input, InputEvent, InputState, Textarea, TextareaState},
    switch::Switch,
};
use knoa_browser::{Browser, Profile};
use knoa_core::{Screening, Workspace, WorkspaceId};

use super::form;
use crate::{
    state::library::{Library, lock},
    ui::{
        components::{Tone, button, workspace_mark},
        theme::Theme,
    },
};
use browsers::synced_from;

pub enum WorkspacePageEvent {
    /// The new workspace exists now, and this page edits it.
    Created(WorkspaceId),
}

impl EventEmitter<WorkspacePageEvent> for WorkspacePage {}

pub struct WorkspacePage {
    /// `None` while creating a workspace.
    existing: Option<WorkspaceId>,
    name: Entity<InputState>,
    instructions: Entity<TextareaState>,
    guideline: Entity<TextareaState>,
    screening: bool,
    color: u8,
    browsers: Vec<String>,
    opens_in: Option<String>,
    profiles: Vec<Profile>,
    blocked: Vec<Browser>,
    /// Delete takes a second click: it removes every bookmark in the workspace.
    confirm_delete: bool,
    deleted: bool,
    library: Entity<Library>,
    _subscriptions: Vec<Subscription>,
}

impl WorkspacePage {
    pub fn new(
        existing: Option<Workspace>,
        library: Entity<Library>,
        window: &mut Window,
        cx: &mut Context<'_, WorkspacePage>,
    ) -> WorkspacePage {
        let is_new = existing.is_none();
        let next_color = library.read(cx).workspaces().len() as u8 % 8;
        let workspace = existing.unwrap_or_else(|| blank_workspace(next_color));
        let name = cx.new(|cx| {
            let mut input = InputState::new(window, cx).placeholder("Work, Personal…");
            input.set_value(workspace.name.clone(), window, cx);
            input
        });
        let instructions = cx.new(|cx| {
            let mut input = TextareaState::new(window, cx)
                .placeholder("e.g. Tag recipes by cuisine. Short summaries, no fluff.");
            input.set_value(workspace.instructions.clone(), window, cx);
            input
        });
        let guideline = cx.new(|cx| {
            let mut input = TextareaState::new(window, cx)
                .placeholder("e.g. Only things about work: engineering, design and our clients.");
            input.set_value(workspace.screening.guideline.clone(), window, cx);
            input
        });
        if is_new {
            name.update(cx, |input, cx| input.focus(window, cx));
        }
        let subscriptions = vec![
            cx.subscribe(&name, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Blur | InputEvent::PressEnter { .. }) {
                    this.save(cx);
                }
            }),
            cx.subscribe(&instructions, Self::save_on_blur),
            cx.subscribe(&guideline, Self::save_on_blur),
            // The header shows the name as it's typed.
            cx.observe(&name, |_, _, cx| cx.notify()),
        ];
        WorkspacePage {
            existing: (!is_new).then_some(workspace.id),
            name,
            instructions,
            guideline,
            screening: workspace.screening.enabled,
            color: workspace.color,
            browsers: workspace.browsers,
            opens_in: workspace.opens_in,
            profiles: knoa_browser::discover_profiles(),
            blocked: knoa_browser::blocked_browsers(),
            confirm_delete: false,
            deleted: false,
            library,
            _subscriptions: subscriptions,
        }
    }

    /// Stores what changed. Does nothing while creating: that waits for the button.
    pub fn save(&mut self, cx: &mut Context<'_, WorkspacePage>) {
        let Some(id) = self.existing.filter(|_| !self.deleted) else {
            return;
        };
        let Some(stored) = self.stored(id, cx) else {
            return;
        };
        let mut workspace = self.workspace(id, cx);
        // An emptied name field keeps the old name rather than saving none.
        if workspace.name.is_empty() {
            workspace.name.clone_from(&stored.name);
        }
        if workspace != stored {
            self.library.update(cx, |library, cx| {
                library.change(cx, move |store| store.update_workspace(&workspace));
            });
        }
    }

    fn save_on_blur(
        &mut self,
        _: Entity<TextareaState>,
        event: &InputEvent,
        cx: &mut Context<'_, WorkspacePage>,
    ) {
        if matches!(event, InputEvent::Blur) {
            self.save(cx);
        }
    }

    fn changed(&mut self, cx: &mut Context<'_, WorkspacePage>) {
        self.save(cx);
        cx.notify();
    }

    fn stored(&self, id: WorkspaceId, cx: &Context<'_, WorkspacePage>) -> Option<Workspace> {
        self.library
            .read(cx)
            .workspaces()
            .iter()
            .find(|w| w.id == id)
            .cloned()
    }

    fn workspace(&self, id: WorkspaceId, cx: &Context<'_, WorkspacePage>) -> Workspace {
        Workspace {
            id,
            name: self.name.read(cx).value().trim().to_owned(),
            color: self.color,
            browsers: self.browsers.clone(),
            opens_in: self.opens_in.clone(),
            instructions: self.instructions.read(cx).value().trim().to_owned(),
            screening: Screening {
                enabled: self.screening,
                guideline: self.guideline.read(cx).value().trim().to_owned(),
            },
        }
    }

    /// Creates the workspace, switches to it, and turns this page into its page.
    fn create(&mut self, window: &mut Window, cx: &mut Context<'_, WorkspacePage>) {
        if self.name.read(cx).value().trim().is_empty() {
            self.name.update(cx, |input, cx| input.focus(window, cx));
            return;
        }
        let mut workspace = self.workspace(WorkspaceId(0), cx);
        let store = self.library.read(cx).store();
        let task = cx.background_spawn(async move {
            let mut store = lock(&store);
            workspace.id = store.create_workspace(&workspace.name, workspace.color)?;
            store.update_workspace(&workspace)?;
            knoa_store::Result::Ok(workspace.id)
        });
        let library = self.library.clone();
        cx.spawn(async move |this, cx| {
            let id = task.await?;
            library.update(cx, |library, cx| library.set_workspace(id, cx));
            this.update(cx, |this, cx| {
                this.existing = Some(id);
                cx.emit(WorkspacePageEvent::Created(id));
                cx.notify();
            })?;
            anyhow::Ok(())
        })
        .detach_and_log_err(cx);
    }

    fn delete(&mut self, id: WorkspaceId, cx: &mut Context<'_, WorkspacePage>) {
        if !self.confirm_delete {
            self.confirm_delete = true;
            cx.notify();
            return;
        }
        self.deleted = true;
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.delete_workspace(id).map(drop));
        });
    }

    fn toggle_browser(&mut self, id: String, cx: &mut Context<'_, WorkspacePage>) {
        if let Some(at) = self.browsers.iter().position(|b| *b == id) {
            self.browsers.remove(at);
        } else {
            self.browsers.push(id);
        }
        self.changed(cx);
    }
}

impl Render for WorkspacePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        div()
            .flex()
            .flex_col()
            .gap_8()
            .child(self.header(&theme, cx))
            .child(self.identity_group(&theme, cx))
            .child(self.browsers_group(&theme, cx))
            .child(self.opens_in_group(&theme, cx))
            .child(self.organizing_group(&theme, cx))
            .children(self.footer(&theme, cx))
    }
}

impl WorkspacePage {
    fn header(&self, theme: &Theme, cx: &mut Context<'_, WorkspacePage>) -> Div {
        let Some(id) = self.existing else {
            return form::header(
                "New workspace",
                "A separate library, like Work or Personal, with its own browsers and folders.",
                theme,
            );
        };
        let typed = self.name.read(cx).value().trim().to_owned();
        let title = if typed.is_empty() {
            self.stored(id, cx).map(|w| w.name).unwrap_or_default()
        } else {
            typed
        };
        let is_current = self.library.read(cx).workspace_id() == id;
        let switch = if is_current {
            div()
                .text_sm()
                .text_color(theme.muted)
                .child("Shown now")
                .into_any_element()
        } else {
            button("show-workspace", "Show it", Tone::Soft, cx)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.save(cx);
                    this.library
                        .update(cx, |library, cx| library.set_workspace(id, cx));
                }))
                .into_any_element()
        };
        div()
            .flex()
            .items_start()
            .gap_4()
            .child(workspace_mark(&title, self.color, px(40.), cx))
            .child(div().flex_1().min_w_0().child(form::header(
                title,
                &synced_from(&self.browsers),
                theme,
            )))
            .child(div().pt_1().child(switch))
    }

    fn identity_group(&self, theme: &Theme, cx: &mut Context<'_, WorkspacePage>) -> Div {
        let mut swatches = div().flex().gap_2();
        for index in 0..8u8 {
            let selected = index == self.color;
            swatches = swatches.child(
                div()
                    .id(("workspace-color", usize::from(index)))
                    .size(px(22.))
                    .rounded(px(7.))
                    .border_2()
                    .border_color(if selected {
                        theme.text
                    } else {
                        gpui::transparent_black()
                    })
                    .p(px(2.))
                    .cursor_pointer()
                    .child(
                        div()
                            .size_full()
                            .rounded(px(4.))
                            .bg(theme.folder_color(index)),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.color = index;
                        this.changed(cx);
                    })),
            );
        }
        let rows = vec![
            form::row(
                "Name",
                None,
                div().w(px(240.)).child(Input::new(&self.name)),
                theme,
            ),
            form::row("Color", None, swatches, theme),
        ];
        form::group("Name and color", rows, theme)
    }

    fn organizing_group(&self, theme: &Theme, cx: &mut Context<'_, WorkspacePage>) -> Div {
        let mut rows = vec![
            form::stacked(
                "Instructions",
                Some("How the AI should sort, tag and summarize what lands here."),
                Textarea::new(&self.instructions).h(px(84.)),
                theme,
            ),
            form::row(
                "Only keep bookmarks that fit",
                Some(
                    "Off: everything from the browser lands here. On: the AI sets aside what doesn't fit, and you can keep it anyway.",
                ),
                Switch::new("screening")
                    .checked(self.screening)
                    .on_click(cx.listener(|this, on: &bool, _, cx| {
                        this.screening = *on;
                        this.changed(cx);
                    })),
                theme,
            ),
        ];
        if self.screening {
            rows.push(form::stacked(
                "What belongs here",
                None,
                Textarea::new(&self.guideline).h(px(84.)),
                theme,
            ));
        }
        form::group("Organizing", rows, theme)
    }

    /// Create for a new workspace; Delete for an existing one, unless it's the last.
    fn footer(&self, theme: &Theme, cx: &mut Context<'_, WorkspacePage>) -> Option<Div> {
        let Some(id) = self.existing else {
            return Some(
                div().flex().justify_end().child(
                    button("create-workspace", "Create workspace", Tone::Primary, cx)
                        .on_click(cx.listener(|this, _, window, cx| this.create(window, cx))),
                ),
            );
        };
        if self.library.read(cx).workspaces().len() < 2 {
            return None;
        }
        let label = if self.confirm_delete {
            "Delete for good"
        } else {
            "Delete…"
        };
        let delete = button("delete-workspace", label, Tone::Soft, cx)
            .when(self.confirm_delete, |s| s.text_color(theme.danger))
            .on_click(cx.listener(move |this, _, _, cx| this.delete(id, cx)));
        let row = form::row(
            "Delete this workspace",
            Some("Its bookmarks and folders go with it. Your browsers keep their bookmarks."),
            delete,
            theme,
        );
        Some(form::group("Delete", vec![row], theme))
    }
}

fn blank_workspace(color: u8) -> Workspace {
    Workspace {
        id: WorkspaceId(0),
        name: String::new(),
        color,
        browsers: Vec::new(),
        opens_in: None,
        instructions: String::new(),
        screening: Screening::default(),
    }
}
