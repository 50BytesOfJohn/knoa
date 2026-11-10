//! The "organize again" row: queues bookmarks that were already organized, so new
//! folders and changed instructions reach them. Either just their folders or everything.
//! Both cost AI credit, so each takes a second click.

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Task, Window, div, prelude::FluentBuilder as _,
};
use knoa_core::WorkspaceId;

use super::form;
use crate::{
    services::organizer::Organizer,
    state::library::Library,
    ui::{
        components::{Tone, button},
        theme::Theme,
    },
};

/// Which bookmarks go back on the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Workspace(WorkspaceId),
    AllWorkspaces,
}

/// How much of the organizing runs again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Only the folder classifier, from what the bookmark already has.
    Folders,
    /// Fetch, kind, summary, fields, tags and folders.
    Everything,
}

impl Mode {
    fn id(self) -> &'static str {
        match self {
            Mode::Folders => "reorganize-folders",
            Mode::Everything => "reorganize-everything",
        }
    }

    fn label(self, confirming: bool) -> &'static str {
        match (self, confirming) {
            (Mode::Folders, false) => "Folders only…",
            (Mode::Folders, true) => "File them all",
            (Mode::Everything, false) => "Everything…",
            (Mode::Everything, true) => "Organize them all",
        }
    }
}

pub struct Reorganize {
    scope: Scope,
    library: Entity<Library>,
    confirming: Option<Mode>,
    /// How many bookmarks the last click queued.
    queued: Option<usize>,
    queuing: Option<Task<()>>,
}

impl Reorganize {
    pub fn new(scope: Scope, library: Entity<Library>) -> Reorganize {
        Reorganize {
            scope,
            library,
            confirming: None,
            queued: None,
            queuing: None,
        }
    }

    fn scope_key(&self) -> usize {
        match self.scope {
            Scope::Workspace(id) => usize::try_from(id.0).unwrap_or_default(),
            Scope::AllWorkspaces => 0,
        }
    }

    fn click(&mut self, mode: Mode, cx: &mut Context<'_, Reorganize>) {
        if self.confirming != Some(mode) {
            self.confirming = Some(mode);
            cx.notify();
            return;
        }
        self.confirming = None;
        let (database, scope) = (self.library.read(cx).database(), self.scope);
        let task = cx.background_spawn(async move {
            let mut store = database.write();
            let workspaces = match scope {
                Scope::Workspace(id) => vec![id],
                Scope::AllWorkspaces => store.workspaces()?.into_iter().map(|w| w.id).collect(),
            };
            workspaces.into_iter().try_fold(0, |total, id| {
                let queued = match mode {
                    Mode::Folders => store.queue_folders(id)?,
                    Mode::Everything => store.queue_all(id)?,
                };
                Ok(total + queued)
            })
        });
        let library = self.library.clone();
        self.queuing = Some(cx.spawn(async move |this, cx| {
            let queued: knoa_store::Result<usize> = task.await;
            library.update(cx, Library::refresh);
            cx.update(Organizer::wake);
            let _ = this.update(cx, |this, cx| {
                match queued {
                    Ok(count) => this.queued = Some(count),
                    Err(err) => tracing::error!("can't queue bookmarks to organize again: {err}"),
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn mode_button(
        &self,
        mode: Mode,
        theme: &Theme,
        cx: &mut Context<'_, Self>,
    ) -> impl IntoElement {
        let confirming = self.confirming == Some(mode);
        button(
            (mode.id(), self.scope_key()),
            mode.label(confirming),
            Tone::Soft,
            cx,
        )
        .when(confirming, |s| s.text_color(theme.ai))
        .on_click(cx.listener(move |this, _, _, cx| this.click(mode, cx)))
    }
}

impl Render for Reorganize {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let theme = &theme;
        let (label, about) = match self.scope {
            Scope::Workspace(_) => (
                "Organize again",
                "Folders only files organized bookmarks into the current folders, without reading pages or asking the chat model again. Everything also redoes summaries, fields and tags with the current instructions. Folders you set or removed stay as they are.",
            ),
            Scope::AllWorkspaces => (
                "Organize every workspace again",
                "The same for all workspaces at once, after you've changed folders in several.",
            ),
        };
        let about = match self.queued {
            Some(count) => format!("Queued {count} bookmarks."),
            None => about.to_owned(),
        };
        let controls = div()
            .flex()
            .gap_2()
            .child(self.mode_button(Mode::Folders, theme, cx))
            .child(self.mode_button(Mode::Everything, theme, cx));
        form::row(label, Some(&about), controls, theme)
    }
}
