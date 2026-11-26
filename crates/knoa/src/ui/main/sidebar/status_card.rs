//! The status card above the footer: one message from the AI or sync with the action that
//! fixes it, and a progress bar while organizing. Nothing at all while all is well.

use gpui::{
    Animation, AnimationExt as _, AnyElement, App, Context, FontWeight, Hsla,
    InteractiveElement as _, IntoElement, ParentElement as _, Pixels, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, ease_out_quint, prelude::FluentBuilder as _,
    px, relative,
};

use super::{
    INSET, SidebarEvent, SidebarView,
    status::{CloudState, Status},
};
use knoa_core::Language;

use crate::{
    assets::line_icon,
    i18n::{self, t},
    services::cloud::{Cloud, Phase},
    ui::{
        components::svg_icon,
        main::settings::Page,
        mascot::{Mood, mascot},
        motion,
        theme::Theme,
    },
};

const MASCOT: Pixels = px(18.);
const SYNC_ICON: Pixels = px(16.);
const BAR_HEIGHT: Pixels = px(3.);

/// The card's message, worked out when the status changes rather than on every frame.
#[derive(Default)]
pub(super) struct StatusCard {
    /// The message in the language it was written in, so a switch writes it again.
    shown: Option<(Status, Language, Message)>,
    /// Goes up each time the card appears, which replays its fade.
    appearances: usize,
    bar: Bar,
}

/// The bar's last move, from one length to the next, so it eases rather than jumps.
#[derive(Default)]
struct Bar {
    from: f32,
    to: f32,
    moves: usize,
}

struct Message {
    text: SharedString,
    tone: Tone,
    mark: Mark,
    action: Option<(SharedString, CardAction)>,
}

#[derive(Clone, Copy)]
enum Tone {
    Danger,
    Ai,
    Plain,
}

#[derive(Clone, Copy)]
enum Mark {
    Mascot(Mood),
    CloudAlert,
    CloudOff,
}

#[derive(Clone, Copy)]
enum CardAction {
    OpenAiSettings,
    RetrySync,
    OpenAccountSettings,
    Upgrade,
}

impl StatusCard {
    fn show(&mut self, status: Option<Status>) -> Option<&Message> {
        let Some(status) = status else {
            self.shown = None;
            self.bar = Bar::default();
            return None;
        };
        let language = i18n::language();
        let key = self.shown.as_ref().map(|(shown, lang, _)| (*shown, *lang));
        if key != Some((status, language)) {
            if self.shown.is_none() {
                self.appearances += 1;
            }
            if let Status::Organizing(progress) = status {
                self.bar = Bar {
                    from: self.bar.to,
                    to: progress.fraction,
                    moves: self.bar.moves + 1,
                };
            }
            self.shown = Some((status, language, Message::of(status)));
        }
        self.shown.as_ref().map(|(_, _, message)| message)
    }
}

impl Message {
    fn of(status: Status) -> Message {
        let message = |text: String, tone, mark, action: Option<(String, CardAction)>| Message {
            text: text.into(),
            tone,
            mark,
            action: action.map(|(label, action)| (label.into(), action)),
        };
        match status {
            Status::AiNeedsAttention => message(
                t!("sidebar-status-ai-needs-attention"),
                Tone::Danger,
                Mark::Mascot(Mood::Worried),
                Some((t!("sidebar-status-fix"), CardAction::OpenAiSettings)),
            ),
            Status::AllowanceUsedUp => message(
                t!("sidebar-status-ai-needs-attention"),
                Tone::Danger,
                Mark::Mascot(Mood::Worried),
                Some((t!("sidebar-status-fix"), CardAction::OpenAccountSettings)),
            ),
            Status::SyncStopped => message(
                t!("sidebar-sync-stopped"),
                Tone::Danger,
                Mark::CloudAlert,
                Some((t!("sidebar-retry"), CardAction::RetrySync)),
            ),
            Status::SyncPaused => message(
                t!("sidebar-sync-paused"),
                Tone::Danger,
                Mark::CloudAlert,
                Some((t!("sidebar-status-fix"), CardAction::OpenAccountSettings)),
            ),
            Status::ChangesNotSynced => message(
                t!("sidebar-status-changes-not-synced"),
                Tone::Plain,
                Mark::CloudAlert,
                Some((t!("sidebar-status-view"), CardAction::OpenAccountSettings)),
            ),
            Status::PlanLimit => message(
                t!("sidebar-status-plan-limit"),
                Tone::Plain,
                Mark::CloudAlert,
                Some((t!("sidebar-status-upgrade"), CardAction::Upgrade)),
            ),
            Status::Organizing(progress) => Message {
                text: t!(
                    "sidebar-status-organizing",
                    current = (progress.current),
                    total = (progress.total)
                )
                .into(),
                tone: Tone::Ai,
                mark: Mark::Mascot(Mood::Organizing),
                action: None,
            },
            Status::FreeAllowanceUsedUp => message(
                t!("sidebar-status-free-allowance-used-up"),
                Tone::Plain,
                Mark::Mascot(Mood::Asleep),
                Some((t!("sidebar-status-upgrade"), CardAction::Upgrade)),
            ),
            Status::AiOff => message(
                t!("sidebar-status-ai-off"),
                Tone::Plain,
                Mark::Mascot(Mood::Asleep),
                // AI is off only while no provider is set up, so setting one up turns it on.
                Some((t!("sidebar-status-turn-on"), CardAction::OpenAiSettings)),
            ),
            Status::Offline => message(
                t!("sidebar-status-offline"),
                Tone::Plain,
                Mark::CloudOff,
                None,
            ),
        }
    }
}

impl SidebarView {
    pub(super) fn status_card(&mut self, cx: &mut Context<'_, SidebarView>) -> Option<AnyElement> {
        let status = self.current_status(cx);
        let organizing = matches!(status, Some(Status::Organizing(_)));
        let message = self.status.show(status)?;
        let (text, tone, mark_kind, action) = (
            message.text.clone(),
            message.tone,
            message.mark,
            message.action.clone(),
        );
        let theme = Theme::get(cx);
        let row =
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .min_h(px(24.))
                .text_size(px(12.5))
                .child(mark(mark_kind, theme))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_color(tone_color(tone, theme))
                        .child(text),
                )
                .when_some(action, |s, (label, action)| {
                    s.child(text_button("status-card-action", label, theme).on_click(
                        cx.listener(move |this, _, _, cx| this.run_card_action(action, cx)),
                    ))
                });
        let card = div()
            .flex_none()
            .mx(INSET)
            .mb(INSET)
            .pt(px(9.))
            .pb(px(9.))
            .pl(px(9.))
            .pr(px(8.))
            .rounded(px(10.))
            .bg(theme.soft)
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(row)
            .when(organizing, |s| s.child(self.progress_bar(cx)));
        let fade = Animation::new(motion::CARD_FADE);
        Some(
            card.with_animation(
                ("status-card", self.status.appearances),
                fade,
                gpui::Styled::opacity,
            )
            .into_any_element(),
        )
    }

    fn current_status(&self, cx: &App) -> Option<Status> {
        let cloud = self.cloud.read(cx);
        let notices = cloud.notices();
        let cloud_state = match cloud.phase() {
            Phase::SignedIn(_) => CloudState::SignedIn {
                sync: cloud.sync(),
                notices: &notices,
            },
            Phase::OtherAccount(_) => CloudState::OtherAccount,
            Phase::SignedOut | Phase::SigningIn(_) => CloudState::Local,
        };
        let batch = self.library.read(cx).batch().progress();
        Status::of(self.organizer.read(cx).status(), cloud_state, batch)
    }

    fn progress_bar(&self, cx: &App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let Bar { from, to, moves } = self.status.bar;
        let fill = div().h_full().rounded(px(2.)).bg(theme.ai);
        let ease = Animation::new(motion::BAR).with_easing(ease_out_quint());
        div()
            .h(BAR_HEIGHT)
            .rounded(px(2.))
            .bg(theme.hairline)
            .overflow_hidden()
            .child(
                fill.with_animation(("status-bar", moves), ease, move |fill, delta| {
                    fill.w(relative(from + (to - from) * delta))
                }),
            )
    }

    fn run_card_action(&mut self, action: CardAction, cx: &mut Context<'_, SidebarView>) {
        match action {
            CardAction::OpenAiSettings => cx.emit(SidebarEvent::OpenSettings(Page::Ai)),
            CardAction::OpenAccountSettings => {
                cx.emit(SidebarEvent::OpenSettings(Page::Account));
            }
            CardAction::RetrySync => self.cloud.update(cx, Cloud::retry_sync),
            CardAction::Upgrade => {
                let pricing = self.cloud.read(cx).config().map(|c| c.pricing.clone());
                if let Some(url) = pricing {
                    cx.open_url(url.as_str());
                }
            }
        }
    }
}

fn mark(mark: Mark, theme: &Theme) -> AnyElement {
    match mark {
        // Asleep, the mascot is grey: the AI's red is for when it works or needs you.
        Mark::Mascot(Mood::Asleep) => mascot(
            "status-mascot",
            MASCOT,
            Mood::Asleep,
            theme.faint,
            theme.soft,
        ),
        // A new id per mood replays that mood's motion from its start.
        Mark::Mascot(mood) => mascot(
            ("status-mascot", mood as usize),
            MASCOT,
            mood,
            theme.ai,
            theme.soft,
        ),
        Mark::CloudAlert => {
            svg_icon(line_icon!("cloud-alert"), SYNC_ICON, theme.danger).into_any_element()
        }
        Mark::CloudOff => {
            svg_icon(line_icon!("cloud-off"), SYNC_ICON, theme.muted).into_any_element()
        }
    }
}

fn tone_color(tone: Tone, theme: &Theme) -> Hsla {
    match tone {
        Tone::Danger => theme.danger,
        Tone::Ai => theme.ai_text,
        Tone::Plain => theme.text,
    }
}

/// A small accent-coloured text button, for the card's action and the account menu's.
pub(super) fn text_button(
    id: &'static str,
    label: impl Into<SharedString>,
    theme: &Theme,
) -> gpui::Stateful<gpui::Div> {
    let hover = theme.hover;
    div()
        .id(id)
        .flex_none()
        .h(px(24.))
        .px(px(6.))
        .flex()
        .items_center()
        .rounded(px(6.))
        .text_size(px(12.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.accent)
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .child(label.into())
}
