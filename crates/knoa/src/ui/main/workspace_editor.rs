//! Creating and editing a workspace: name and colour, the browsers it syncs, how it's
//! organized, and an optional guideline for what it keeps.

use gpui::{
    AppContext as _, Context, Entity, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _, TaskExt as _,
    WeakEntity, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    input::{Input, InputState, Textarea, TextareaState},
    switch::Switch,
};
use knoa_browser::Profile;
use knoa_core::{Screening, Workspace, WorkspaceId};

use super::MainView;
use crate::{
    state::library::{Library, lock},
    ui::{
        components::{Tone, button, workspace_mark},
        theme::{SERIF, Theme},
    },
};

pub struct WorkspaceEditor {
    existing: Option<WorkspaceId>,
    name: Entity<InputState>,
    instructions: Entity<TextareaState>,
    guideline: Entity<TextareaState>,
    screening: bool,
    color: u8,
    browsers: Vec<String>,
    profiles: Vec<Profile>,
    /// Delete takes a second click: it removes every bookmark in the workspace.
    confirm_delete: bool,
    library: Entity<Library>,
    main: WeakEntity<MainView>,
}

impl WorkspaceEditor {
    pub fn new(
        existing: Option<Workspace>,
        library: Entity<Library>,
        main: WeakEntity<MainView>,
        window: &mut Window,
        cx: &mut Context<'_, WorkspaceEditor>,
    ) -> WorkspaceEditor {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("Workspace name"));
        let instructions = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Optional. e.g. Tag recipes by cuisine. Short summaries, no fluff.")
        });
        let guideline = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("e.g. Only things about work: engineering, design and our clients.")
        });
        let next_color = library.read(cx).workspaces().len() as u8 % 8;
        let workspace = existing.unwrap_or_else(|| Workspace {
            id: WorkspaceId(0),
            name: String::new(),
            color: next_color,
            browsers: Vec::new(),
            instructions: String::new(),
            screening: Screening::default(),
        });
        let is_new = workspace.id == WorkspaceId(0);
        name.update(cx, |input, cx| {
            input.set_value(workspace.name.clone(), window, cx);
            input.focus(window, cx);
        });
        instructions.update(cx, |input, cx| {
            input.set_value(workspace.instructions.clone(), window, cx);
        });
        guideline.update(cx, |input, cx| {
            input.set_value(workspace.screening.guideline.clone(), window, cx);
        });
        WorkspaceEditor {
            existing: (!is_new).then_some(workspace.id),
            name,
            instructions,
            guideline,
            screening: workspace.screening.enabled,
            color: workspace.color,
            browsers: workspace.browsers,
            profiles: knoa_browser::discover_profiles(),
            confirm_delete: false,
            library,
            main,
        }
    }

    fn workspace(&self, id: WorkspaceId, cx: &Context<'_, WorkspaceEditor>) -> Workspace {
        Workspace {
            id,
            name: self.name.read(cx).value().trim().to_owned(),
            color: self.color,
            browsers: self.browsers.clone(),
            instructions: self.instructions.read(cx).value().trim().to_owned(),
            screening: Screening {
                enabled: self.screening,
                guideline: self.guideline.read(cx).value().trim().to_owned(),
            },
        }
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<'_, WorkspaceEditor>) {
        if self.name.read(cx).value().trim().is_empty() {
            return;
        }
        match self.existing {
            Some(id) => {
                let workspace = self.workspace(id, cx);
                self.library.update(cx, |library, cx| {
                    library.change(cx, move |store| store.update_workspace(&workspace));
                });
            }
            None => self.create(cx),
        }
        self.close(window, cx);
    }

    /// Creates the workspace and switches to it.
    fn create(&self, cx: &mut Context<'_, WorkspaceEditor>) {
        let mut workspace = self.workspace(WorkspaceId(0), cx);
        let store = self.library.read(cx).store();
        let task = cx.background_spawn(async move {
            let mut store = lock(&store);
            workspace.id = store.create_workspace(&workspace.name, workspace.color)?;
            store.update_workspace(&workspace)?;
            knoa_store::Result::Ok(workspace.id)
        });
        let library = self.library.clone();
        cx.spawn(async move |_, cx| {
            let id = task.await?;
            library.update(cx, |library, cx| library.set_workspace(id, cx));
            anyhow::Ok(())
        })
        .detach_and_log_err(cx);
    }

    fn delete(&mut self, id: WorkspaceId, window: &mut Window, cx: &mut Context<'_, Self>) {
        if !self.confirm_delete {
            self.confirm_delete = true;
            cx.notify();
            return;
        }
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.delete_workspace(id).map(drop));
        });
        self.close(window, cx);
    }

    fn toggle_browser(&mut self, id: String, cx: &mut Context<'_, WorkspaceEditor>) {
        if let Some(at) = self.browsers.iter().position(|b| *b == id) {
            self.browsers.remove(at);
        } else {
            self.browsers.push(id);
        }
        cx.notify();
    }

    fn close(&self, window: &mut Window, cx: &mut Context<'_, WorkspaceEditor>) {
        let _ = self
            .main
            .update(cx, |main, cx| main.close_overlay(window, cx));
    }
}

impl Render for WorkspaceEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let title = if self.existing.is_some() {
            "Edit workspace"
        } else {
            "New workspace"
        };
        div()
            .id("workspace-editor")
            .w(px(440.))
            .max_h(px(660.))
            .overflow_y_scroll()
            .p_5()
            .flex()
            .flex_col()
            .gap_5()
            .child(
                div()
                    .font_family(SERIF)
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title),
            )
            .child(self.identity(cx))
            .child(self.browsers_section(cx))
            .child(self.organizing_section(cx))
            .child(self.screening_section(cx))
            .child(self.buttons(cx))
    }
}

impl WorkspaceEditor {
    fn identity(&self, cx: &mut Context<'_, WorkspaceEditor>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let name = self.name.read(cx).value().to_string();
        let mut swatches = div().flex().gap_2();
        for index in 0..8u8 {
            swatches = swatches.child(
                div()
                    .id(("workspace-color", usize::from(index)))
                    .size(px(22.))
                    .rounded(px(7.))
                    .bg(theme.folder_color(index))
                    .cursor_pointer()
                    .when(index == self.color, |s| {
                        s.border_2().border_color(theme.text)
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.color = index;
                        cx.notify();
                    })),
            );
        }
        let mark_name = if name.trim().is_empty() { "?" } else { &name };
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(workspace_mark(mark_name, self.color, px(36.), cx))
                    .child(div().flex_1().child(Input::new(&self.name))),
            )
            .child(swatches)
    }

    fn browsers_section(&self, cx: &mut Context<'_, WorkspaceEditor>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let mut list = div().flex().flex_col().gap_2();
        for (index, profile) in self.profiles.iter().enumerate() {
            let id = profile.id.to_string();
            list = list.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_sm()
                    .child(div().flex_1().child(format!(
                        "{} · {}",
                        profile.id.browser.name(),
                        profile.name
                    )))
                    .child(
                        Switch::new(("workspace-profile", index))
                            .checked(self.browsers.contains(&id))
                            .on_click(cx.listener(move |this, _: &bool, _, cx| {
                                this.toggle_browser(id.clone(), cx);
                            })),
                    ),
            );
        }
        if self.profiles.is_empty() {
            list = list.child(hint(
                "No Chromium-based browser found. Knoa reads Helium, Chrome, Brave, Edge, Vivaldi and Chromium.",
                &theme,
            ));
        }
        section("Sync bookmarks from", &theme)
            .child(list)
            .child(hint(
                "Pick none to add links by hand only (⌘N). A browser can feed several workspaces.",
                &theme,
            ))
    }

    fn organizing_section(&self, cx: &mut Context<'_, WorkspaceEditor>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        section("How should things be organized here?", &theme)
            .child(Textarea::new(&self.instructions).h(px(76.)))
    }

    fn screening_section(&self, cx: &mut Context<'_, WorkspaceEditor>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let toggle = div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(div().text_sm().child("Only keep bookmarks that fit"))
                    .child(hint(
                        "Off: everything from the browser lands here. On: the AI sets aside what doesn't fit, and you can keep it anyway.",
                        &theme,
                    )),
            )
            .child(
                Switch::new("screening")
                    .checked(self.screening)
                    .on_click(cx.listener(|this, on: &bool, _, cx| {
                        this.screening = *on;
                        cx.notify();
                    })),
            );
        section("What belongs here", &theme)
            .child(toggle)
            .when(self.screening, |s| {
                s.child(Textarea::new(&self.guideline).h(px(76.)))
            })
    }

    fn buttons(&self, cx: &mut Context<'_, WorkspaceEditor>) -> impl IntoElement {
        let deletable = self
            .existing
            .filter(|_| self.library.read(cx).workspaces().len() > 1);
        let delete_label = if self.confirm_delete {
            "Delete it and its bookmarks?"
        } else {
            "Delete"
        };
        div()
            .flex()
            .gap_2()
            .when_some(deletable, |s, id| {
                s.child(
                    button("delete-workspace", delete_label, Tone::Ghost, cx).on_click(
                        cx.listener(move |this, _, window, cx| this.delete(id, window, cx)),
                    ),
                )
            })
            .child(div().flex_1())
            .child(
                button("cancel-workspace", "Cancel", Tone::Ghost, cx)
                    .on_click(cx.listener(|this, _, window, cx| this.close(window, cx))),
            )
            .child(
                button("save-workspace", "Save", Tone::Primary, cx)
                    .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
            )
    }
}

fn section(title: &str, theme: &Theme) -> gpui::Div {
    div().flex().flex_col().gap_2().child(
        div()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.faint)
            .child(title.to_owned()),
    )
}

fn hint(text: &str, theme: &Theme) -> gpui::Div {
    div()
        .text_xs()
        .text_color(theme.muted)
        .child(text.to_owned())
}
