//! The "organize again" row: queues bookmarks that were already organized, so new
//! folders and changed instructions reach them. It costs AI credit, so it takes a second
//! click.

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Task, Window, div, prelude::FluentBuilder as _,
};
use knoa_core::WorkspaceId;

use super::form;
use crate::{
    services::organizer::Organizer,
    state::library::{Library, lock},
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

pub struct Reorganize {
    scope: Scope,
    library: Entity<Library>,
    confirming: bool,
    /// How many bookmarks the last click queued.
    queued: Option<usize>,
    queuing: Option<Task<()>>,
}

impl Reorganize {
    pub fn new(scope: Scope, library: Entity<Library>) -> Reorganize {
        Reorganize {
            scope,
            library,
            confirming: false,
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

    fn click(&mut self, cx: &mut Context<'_, Reorganize>) {
        if !self.confirming {
            self.confirming = true;
            cx.notify();
            return;
        }
        self.confirming = false;
        let (store, scope) = (self.library.read(cx).store(), self.scope);
        let task = cx.background_spawn(async move {
            let mut store = lock(&store);
            let workspaces = match scope {
                Scope::Workspace(id) => vec![id],
                Scope::AllWorkspaces => store.workspaces()?.into_iter().map(|w| w.id).collect(),
            };
            workspaces
                .into_iter()
                .try_fold(0, |total, id| Ok(total + store.queue_all(id)?))
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
}

impl Render for Reorganize {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let theme = &theme;
        let (label, about) = match self.scope {
            Scope::Workspace(_) => (
                "Organize everything again",
                "Files every bookmark here again with the current folders and instructions. Folders you set or removed stay as they are.",
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
        let action = if self.confirming {
            "Organize them all"
        } else {
            "Organize again…"
        };
        let control = button(("reorganize", self.scope_key()), action, Tone::Soft, cx)
            .when(self.confirming, |s| s.text_color(theme.ai))
            .on_click(cx.listener(|this, _, _, cx| this.click(cx)));
        form::row(label, Some(&about), div().child(control), theme)
    }
}
