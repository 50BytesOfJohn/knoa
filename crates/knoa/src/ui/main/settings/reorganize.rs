//! The "organize again" row: queues bookmarks that were already organized, so new
//! folders, changed instructions or a new language reach them. Either just their folders
//! or everything. Both cost AI credit, so each takes a second click.

use gpui::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Subscription, Task, Window, div,
    prelude::FluentBuilder as _,
};
use knoa_core::WorkspaceId;

use super::form;
use crate::{
    i18n::t,
    services::{cloud::Cloud, organizer::Organizer},
    state::{library::Library, settings::SettingsFile},
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

    fn label(self, offer: Offer, confirming: bool) -> String {
        match (self, offer, confirming) {
            (Mode::Folders, _, false) => t!("settings-reorganize-folders"),
            (Mode::Folders, _, true) => t!("settings-reorganize-folders-confirm"),
            (Mode::Everything, Offer::Both, false) => t!("settings-reorganize-everything"),
            (Mode::Everything, Offer::Both, true) => t!("settings-reorganize-everything-confirm"),
            (Mode::Everything, Offer::NewLanguage, false) => {
                t!("settings-general-language-organize")
            }
            (Mode::Everything, Offer::NewLanguage, true) => {
                t!("settings-general-language-organize-confirm")
            }
        }
    }
}

/// What the row offers, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Offer {
    /// Folders only, or everything.
    Both,
    /// Everything, after the language changed: only then are summaries and tags rewritten.
    NewLanguage,
}

impl Offer {
    fn modes(self) -> &'static [Mode] {
        match self {
            Offer::Both => &[Mode::Folders, Mode::Everything],
            Offer::NewLanguage => &[Mode::Everything],
        }
    }
}

pub struct Reorganize {
    scope: Scope,
    offer: Offer,
    library: Entity<Library>,
    confirming: Option<Mode>,
    /// How many bookmarks the last click queued.
    queued: Option<usize>,
    queuing: Option<Task<()>>,
    _watching: Vec<Subscription>,
}

impl Reorganize {
    pub fn new(scope: Scope, library: Entity<Library>) -> Reorganize {
        Reorganize::offering(Offer::Both, scope, library)
    }

    /// The note under the language setting: what a new language leaves as it was, and
    /// organizing every workspace again to rewrite it. When the cloud organizes, that
    /// waits until the account has the new language.
    pub fn after_language_change(
        library: Entity<Library>,
        cx: &mut Context<'_, Reorganize>,
    ) -> Reorganize {
        let mut note = Reorganize::offering(Offer::NewLanguage, Scope::AllWorkspaces, library);
        note._watching = vec![
            cx.observe(&Cloud::entity(cx), |_, _, cx| cx.notify()),
            cx.observe_global::<SettingsFile>(|_, cx| cx.notify()),
        ];
        note
    }

    fn offering(offer: Offer, scope: Scope, library: Entity<Library>) -> Reorganize {
        Reorganize {
            scope,
            offer,
            library,
            confirming: None,
            queued: None,
            queuing: None,
            _watching: Vec::new(),
        }
    }

    /// Whether the new language is still on its way to an account whose cloud organizes.
    fn waits_for_language(&self, cx: &App) -> bool {
        self.offer == Offer::NewLanguage
            && Cloud::entity(cx).read(cx).organizer_waits_for_language(cx)
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
            mode.label(self.offer, confirming),
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
        let (label, about) = match (self.offer, self.scope) {
            (Offer::NewLanguage, _) => (t!("settings-general-language-note"), None),
            (Offer::Both, Scope::Workspace(_)) => (
                t!("settings-reorganize-title"),
                Some(t!("settings-reorganize-about")),
            ),
            (Offer::Both, Scope::AllWorkspaces) => (
                t!("settings-reorganize-all-title"),
                Some(t!("settings-reorganize-all-about")),
            ),
        };
        let waiting = self.waits_for_language(cx);
        let about = match self.queued {
            Some(count) => Some(t!("settings-reorganize-queued", count = count)),
            None if waiting => Some(t!("settings-general-language-waiting")),
            None => about,
        };
        let modes = if waiting { &[] } else { self.offer.modes() };
        let controls = modes.iter().fold(div().flex().gap_2(), |row, &mode| {
            row.child(self.mode_button(mode, theme, cx))
        });
        form::row(&label, about.as_deref(), controls, theme)
    }
}
