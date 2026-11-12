//! A workspace's page: name and color, the browsers it syncs and opens links in, how it's
//! organized, an optional guideline for what it keeps, and its tag list. The same page
//! creates one.
//!
//! An existing workspace saves as you go, like the rest of settings: switches and colors
//! at once, text when its field loses focus or the page is left.

mod browsers;
mod identity;
mod organizing;
mod tag_list;

use gpui::{
    AppContext as _, Context, Div, Entity, EventEmitter, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Subscription, TaskExt as _, Window, div,
    prelude::FluentBuilder as _,
};
use gpui_component::input::{InputEvent, InputState, TextareaState};
use knoa_browser::{Browser, Profile};
use knoa_core::{
    Ink, Screening, TagList, Workspace, WorkspaceId,
    protocol::max_chars::{self, fits},
};

use super::{
    form,
    reorganize::{Reorganize, Scope},
};
use crate::{
    services::organizer::Organizer,
    state::library::Library,
    ui::{
        components::{Tone, button},
        theme::Theme,
    },
};
use tag_list::{TagListChanged, TagListEditor};

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
    tag_list: Entity<TagListEditor>,
    color: Ink,
    browsers: Vec<String>,
    opens_in: Option<String>,
    profiles: Vec<Profile>,
    blocked: Vec<Browser>,
    /// Delete takes a second click: it removes every bookmark in the workspace.
    confirm_delete: bool,
    deleted: bool,
    /// `None` while creating a workspace.
    reorganize: Option<Entity<Reorganize>>,
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
        let next_color = Ink::nth(library.read(cx).workspaces().len());
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
        let tag_list = cx.new(|cx| TagListEditor::new(&workspace.tag_list, window, cx));
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
            cx.subscribe(&tag_list, |this, _, _: &TagListChanged, cx| this.save(cx)),
            // The header shows the name as it's typed, and the limits' messages follow it.
            cx.observe(&name, |_, _, cx| cx.notify()),
            cx.observe(&instructions, |_, _, cx| cx.notify()),
            cx.observe(&guideline, |_, _, cx| cx.notify()),
            // Organizing again is offered only while AI is on.
            cx.observe(&Organizer::entity(cx), |_, _, cx| cx.notify()),
        ];
        let reorganize = (!is_new).then(|| new_reorganize(workspace.id, &library, cx));
        WorkspacePage {
            existing: (!is_new).then_some(workspace.id),
            name,
            instructions,
            guideline,
            screening: workspace.screening.enabled,
            tag_list,
            color: workspace.color,
            browsers: workspace.browsers,
            opens_in: workspace.opens_in,
            profiles: knoa_browser::discover_profiles(),
            blocked: knoa_browser::blocked_browsers(),
            confirm_delete: false,
            deleted: false,
            reorganize,
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
        keep_stored_where_refused(&mut workspace, &stored);
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
            tag_list: self.tag_list.read(cx).tag_list(cx),
        }
    }

    /// Creates the workspace, switches to it, and turns this page into its page.
    fn create(&mut self, window: &mut Window, cx: &mut Context<'_, WorkspacePage>) {
        let mut workspace = self.workspace(WorkspaceId(0), cx);
        if workspace.name.is_empty() || !fits(&workspace.name, max_chars::workspace::NAME) {
            self.name.update(cx, |input, cx| input.focus(window, cx));
            return;
        }
        let blank = blank_workspace(workspace.color);
        keep_stored_where_refused(&mut workspace, &blank);
        let database = self.library.read(cx).database();
        let task = cx.background_spawn(async move {
            let mut store = database.write();
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
                this.reorganize = Some(new_reorganize(id, &this.library, cx));
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
            .child(self.tag_list.clone())
            .children(self.footer(&theme, cx))
    }
}

impl WorkspacePage {
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

fn new_reorganize(
    id: WorkspaceId,
    library: &Entity<Library>,
    cx: &mut Context<'_, WorkspacePage>,
) -> Entity<Reorganize> {
    let library = library.clone();
    cx.new(|_| Reorganize::new(Scope::Workspace(id), library))
}

/// Puts back the stored value of a text field the server would refuse: an emptied name,
/// or text past its limit, which would keep the whole change on this Mac only.
fn keep_stored_where_refused(workspace: &mut Workspace, stored: &Workspace) {
    use max_chars::workspace::{GUIDELINE, INSTRUCTIONS, NAME};
    if workspace.name.is_empty() || !fits(&workspace.name, NAME) {
        workspace.name.clone_from(&stored.name);
    }
    if !fits(&workspace.instructions, INSTRUCTIONS) {
        workspace.instructions.clone_from(&stored.instructions);
    }
    if !fits(&workspace.screening.guideline, GUIDELINE) {
        workspace
            .screening
            .guideline
            .clone_from(&stored.screening.guideline);
    }
}

fn blank_workspace(color: Ink) -> Workspace {
    Workspace {
        id: WorkspaceId(0),
        name: String::new(),
        color,
        browsers: Vec::new(),
        opens_in: None,
        instructions: String::new(),
        screening: Screening::default(),
        tag_list: TagList::default(),
    }
}
