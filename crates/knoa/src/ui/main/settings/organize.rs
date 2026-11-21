//! The Organize page: which workspaces it's about, organizing the bookmarks the AI hasn't
//! reached yet, and organizing the rest again.

use gpui::{
    AppContext as _, Context, Div, Entity, EventEmitter, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Task, Window, div, px,
};
use knoa_core::{Place, Query, WorkspaceId};

use super::{
    form,
    reorganize::{Reorganize, Scope},
};
use crate::{
    format,
    i18n::t,
    services::organizer::{AiStatus, Organizer},
    state::library::{Library, LibraryChanged},
    ui::{
        choice::select,
        components::{Tone, button},
        mascot::{Mood, mascot},
        theme::Theme,
    },
};

/// About $0.17 per 1,000 bookmarks with the default models (enrichment research).
const DOLLARS_PER_BOOKMARK: f64 = 0.000_17;

pub enum OrganizePageEvent {
    /// Organizing needs AI, which is set up on the AI page.
    OpenAi,
}

impl EventEmitter<OrganizePageEvent> for OrganizePage {}

pub struct OrganizePage {
    library: Entity<Library>,
    scope: Scope,
    again: Entity<Reorganize>,
    /// Bookmarks not organized yet in the scope; `None` until counted.
    waiting: Option<usize>,
    counting: Option<Task<()>>,
}

impl OrganizePage {
    pub fn new(library: Entity<Library>, cx: &mut Context<'_, OrganizePage>) -> OrganizePage {
        cx.observe(&Organizer::entity(cx), |_, _, cx| cx.notify())
            .detach();
        cx.subscribe(&library, |this, _, _: &LibraryChanged, cx| this.count(cx))
            .detach();
        let scope = Scope::AllWorkspaces;
        let again = cx.new(|_| Reorganize::new(scope, library.clone()));
        let mut page = OrganizePage {
            library,
            scope,
            again,
            waiting: None,
            counting: None,
        };
        page.count(cx);
        page
    }

    /// Shows the page for one workspace, or all of them.
    pub fn set_scope(&mut self, scope: Scope, cx: &mut Context<'_, OrganizePage>) {
        if self.scope == scope {
            return;
        }
        self.scope = scope;
        let library = self.library.clone();
        self.again = cx.new(|_| Reorganize::new(scope, library));
        self.waiting = None;
        self.count(cx);
        cx.notify();
    }

    fn workspaces(&self, cx: &Context<'_, OrganizePage>) -> Vec<WorkspaceId> {
        match self.scope {
            Scope::Workspace(id) => vec![id],
            Scope::AllWorkspaces => self
                .library
                .read(cx)
                .workspaces()
                .iter()
                .map(|w| w.id)
                .collect(),
        }
    }

    fn count(&mut self, cx: &mut Context<'_, OrganizePage>) {
        let (database, workspaces) = (self.library.read(cx).database(), self.workspaces(cx));
        let task = cx.background_spawn(async move {
            let store = database.read();
            workspaces.into_iter().try_fold(0, |total, id| {
                let query = Query {
                    workspace: Some(id),
                    place: Place::NotOrganized,
                    ..Query::default()
                };
                knoa_store::Result::Ok(total + store.count(&query)?)
            })
        });
        self.counting = Some(cx.spawn(async move |this, cx| {
            let counted = task.await;
            let _ = this.update(cx, |this, cx| match counted {
                Ok(waiting) => {
                    this.waiting = Some(waiting);
                    cx.notify();
                }
                Err(err) => tracing::error!("can't count bookmarks to organize: {err}"),
            });
        }));
    }

    fn organize_waiting(&mut self, cx: &mut Context<'_, OrganizePage>) {
        let workspaces = self.workspaces(cx);
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| {
                workspaces
                    .iter()
                    .try_for_each(|id| store.queue_not_organized(*id).map(drop))
            });
        });
        Organizer::wake(cx);
    }
}

impl Render for OrganizePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let status = Organizer::entity(cx).read(cx).status().clone();
        let organizing = self.library.read(cx).counts().organizing;
        let header = div()
            .flex()
            .items_center()
            .gap_6()
            .child(div().flex_1().child(form::header(
                t!("settings-organize-title"),
                &t!("settings-organize-about"),
                &theme,
            )))
            .child(mascot(
                "organize-page-mascot",
                px(48.),
                Mood::of(&status, organizing),
                theme.ai,
                theme.panel,
            ));
        let page = div().flex().flex_col().gap_8().child(header);
        if status == AiStatus::Off {
            return page.child(self.ai_off_group(&theme, cx));
        }
        let page = if self.library.read(cx).workspaces().len() > 1 {
            page.child(self.scope_group(&theme, cx))
        } else {
            page
        };
        page.child(self.waiting_group(&status, &theme, cx))
            .child(form::group(
                &t!("settings-reorganize-title"),
                vec![self.again.clone().into_any_element()],
                &theme,
            ))
    }
}

impl OrganizePage {
    fn ai_off_group(&self, theme: &Theme, cx: &mut Context<'_, OrganizePage>) -> Div {
        let row = form::row(
            &t!("settings-organize-ai-off"),
            Some(&t!("settings-organize-ai-off-about")),
            button(
                "organize-open-ai",
                t!("settings-organize-open-ai"),
                Tone::Primary,
                cx,
            )
            .on_click(cx.listener(|_, _, _, cx| cx.emit(OrganizePageEvent::OpenAi))),
            theme,
        );
        form::group(&t!("settings-organize-new"), vec![row], theme)
    }

    fn scope_group(&self, theme: &Theme, cx: &mut Context<'_, OrganizePage>) -> Div {
        let mut options = vec![(Scope::AllWorkspaces, t!("settings-organize-scope-all"))];
        options.extend(
            self.library
                .read(cx)
                .workspaces()
                .iter()
                .map(|w| (Scope::Workspace(w.id), w.name.clone())),
        );
        let page = cx.entity().downgrade();
        let picker = select(
            "organize-scope",
            options,
            Some(self.scope),
            false,
            theme,
            move |scope, cx| {
                let _ = page.update(cx, |page, cx| page.set_scope(scope, cx));
            },
        );
        let row = form::row(&t!("settings-organize-scope"), None, picker, theme);
        form::card(vec![row], theme)
    }

    fn waiting_group(
        &self,
        status: &AiStatus,
        theme: &Theme,
        cx: &mut Context<'_, OrganizePage>,
    ) -> Div {
        let row = match self.waiting {
            None => form::note(t!("settings-organize-counting"), theme.muted),
            Some(0) => form::note(t!("settings-organize-none"), theme.muted),
            Some(waiting) => {
                let about = if *status == AiStatus::Cloud {
                    t!("settings-ai-included-in-pro")
                } else {
                    let cost = waiting as f64 * DOLLARS_PER_BOOKMARK;
                    t!("settings-ai-cost", cost = (format::usd(cost, 2)))
                };
                form::row(
                    &t!("settings-organize-waiting", count = waiting),
                    Some(&about),
                    button(
                        "organize-all",
                        t!("settings-ai-organize"),
                        Tone::Primary,
                        cx,
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.organize_waiting(cx))),
                    theme,
                )
            }
        };
        form::group(&t!("settings-organize-new"), vec![row], theme)
    }
}
