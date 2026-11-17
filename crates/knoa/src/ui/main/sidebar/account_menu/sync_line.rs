//! The account menu's line on how sync is doing: "Synced 5 min ago", "Offline · 3 changes
//! waiting", or what stopped it.

use std::time::Duration;

use gpui::{
    AnyElement, Context, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, Task, div, prelude::FluentBuilder as _, px,
};
use jiff::{SignedDuration, Timestamp};

use super::{ITEM_HEIGHT, ITEM_ICON};
use crate::{
    assets::line_icon,
    services::cloud::{Cloud, Phase, SyncStatus},
    ui::{
        components::svg_icon,
        main::sidebar::{SidebarView, status_card::text_button},
        theme::Theme,
    },
};

/// "Synced 5 min ago" for the last sync, kept until the age it shows changes. The menu
/// redraws on every frame of its animations, and the age changes once a minute at most.
#[derive(Default)]
pub(super) struct SyncedAge {
    /// The sync the text is for, and the timer that drops it when the next age is due.
    /// `None` once the text has gone stale.
    shown: Option<(Timestamp, Task<()>)>,
    text: SharedString,
}

/// What the line shows: its icon, its text and colour, and whether it offers a retry.
struct Line {
    icon: SharedString,
    text: SharedString,
    tone: Tone,
    retry: bool,
}

#[derive(Clone, Copy)]
enum Tone {
    Danger,
    Muted,
}

impl Line {
    fn danger(icon: SharedString, text: &'static str) -> Line {
        Line {
            icon,
            text: SharedString::new_static(text),
            tone: Tone::Danger,
            retry: false,
        }
    }

    fn muted(icon: SharedString, text: SharedString) -> Line {
        Line {
            icon,
            text,
            tone: Tone::Muted,
            retry: false,
        }
    }
}

impl SidebarView {
    pub(super) fn sync_line(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let line = self.sync_line_content(cx);
        let theme = Theme::get(cx);
        let color = match line.tone {
            Tone::Danger => theme.danger,
            Tone::Muted => theme.muted,
        };
        div()
            .flex()
            .items_center()
            .gap(px(9.))
            .h(ITEM_HEIGHT)
            .px(px(8.))
            .text_size(px(12.5))
            .text_color(color)
            .child(svg_icon(line.icon, ITEM_ICON, color))
            .child(div().flex_1().min_w_0().truncate().child(line.text))
            .when(line.retry, |s| {
                s.child(
                    text_button("Retry", theme).on_click(cx.listener(|this, _, _, cx| {
                        this.cloud.update(cx, Cloud::retry_sync);
                    })),
                )
            })
            .into_any_element()
    }

    fn sync_line_content(&mut self, cx: &mut Context<'_, Self>) -> Line {
        let cloud = self.cloud.read(cx);
        let paused = matches!(cloud.phase(), Phase::OtherAccount(_));
        let stopped = matches!(cloud.sync(), SyncStatus::Error(_));
        let offline_with = (*cloud.sync() == SyncStatus::Offline).then(|| cloud.waiting_changes());
        let last_synced = cloud.last_synced();
        if paused {
            Line::danger(line_icon!("cloud-alert"), "Sync paused")
        } else if stopped {
            Line {
                retry: true,
                ..Line::danger(line_icon!("cloud-alert"), "Sync stopped")
            }
        } else if let Some(waiting) = offline_with {
            Line::muted(line_icon!("cloud-off"), offline(waiting))
        } else if let Some(at) = last_synced {
            Line::muted(line_icon!("cloud-check"), self.synced_age(at, cx))
        } else {
            Line::muted(line_icon!("refresh-cw"), "Syncing…".into())
        }
    }

    /// The age of the sync at `at`, worked out again only once it can have changed.
    fn synced_age(&mut self, at: Timestamp, cx: &mut Context<'_, Self>) -> SharedString {
        let Some(open) = self.account_menu.open.as_mut() else {
            return synced(at, Timestamp::now());
        };
        let age = &mut open.age;
        if age.shown.as_ref().map(|(of, _)| *of) != Some(at) {
            let now = Timestamp::now();
            age.text = synced(at, now);
            let wait = until_next_age(now.duration_since(at));
            let stale = cx.spawn(async move |this, cx| {
                cx.background_executor().timer(wait).await;
                let _ = this.update(cx, |this, cx| {
                    if let Some(open) = this.account_menu.open.as_mut() {
                        open.age.shown = None;
                        cx.notify();
                    }
                });
            });
            age.shown = Some((at, stale));
        }
        age.text.clone()
    }
}

/// "Offline · 3 changes waiting", or just "Offline" with none.
fn offline(waiting: usize) -> SharedString {
    match waiting {
        0 => "Offline".into(),
        1 => "Offline · 1 change waiting".into(),
        n => format!("Offline · {n} changes waiting").into(),
    }
}

/// "Synced just now", then "1 min ago", "5 min ago", "1 hr ago".
fn synced(at: Timestamp, now: Timestamp) -> SharedString {
    let ago = now.duration_since(at).max(SignedDuration::ZERO);
    let minutes = ago.as_secs() / 60;
    match minutes {
        0 => "Synced just now".into(),
        1..60 => format!("Synced {minutes} min ago").into(),
        _ => format!("Synced {} hr ago", minutes / 60).into(),
    }
}

/// How long until `synced` says something else for a sync `ago` old: the next minute
/// under an hour, the next hour after. Rounded up, so it is never early.
fn until_next_age(ago: SignedDuration) -> Duration {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    let elapsed = Duration::try_from(ago).unwrap_or_default().as_secs();
    let step = if elapsed < HOUR { MINUTE } else { HOUR };
    Duration::from_secs(step - elapsed % step)
}
