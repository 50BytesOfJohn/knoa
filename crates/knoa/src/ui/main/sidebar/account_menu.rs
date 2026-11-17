//! The account menu, opening upward from the footer's account button: the account and how
//! sync is doing, the plan, account settings, the web app and signing out; or, signed out,
//! signing in.
//!
//! It closes on Esc, on a press outside it and after an item is chosen.

use gpui::{
    Animation, AnimationExt as _, AnyElement, App, Context, FocusHandle, FontWeight,
    InteractiveElement as _, IntoElement, KeyBinding, ParentElement as _, Pixels, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, actions, div,
    prelude::FluentBuilder as _, px,
};
use jiff::{SignedDuration, Timestamp};
use knoa_core::protocol::Plan;

use super::{
    INSET, SidebarEvent, SidebarView,
    account::{MENU_AVATAR, Who, avatar},
    footer::FOOTER_HEIGHT,
    status_card::text_button,
};
use crate::{
    assets::line_icon,
    services::cloud::{Cloud, Phase, SyncStatus},
    ui::{components::svg_icon, main::settings::Page, motion, theme::Theme},
};

actions!(account_menu, [CloseAccountMenu]);

const KEY_CONTEXT: &str = "AccountMenu";
const WIDTH: Pixels = px(224.);
/// From the footer's top line to the menu's bottom edge.
const ABOVE_FOOTER: Pixels = px(4.);
const ITEM_HEIGHT: Pixels = px(30.);
const ITEM_ICON: Pixels = px(15.);
const SIGNED_OUT_NOTE: &str =
    "Sign in to sync your library across your devices and open it on the web.";

pub(in crate::ui::main) fn bind_account_menu_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new(
        "escape",
        CloseAccountMenu,
        Some(KEY_CONTEXT),
    )]);
}

/// The menu's part of the sidebar's state.
#[derive(Default)]
pub(super) struct AccountMenu {
    /// Held while the menu is open, so Esc reaches it.
    focus: Option<FocusHandle>,
    /// Goes up with each opening, which replays the opening motion.
    opened: usize,
    /// A press on the account button is under way. It toggles the menu itself, so the
    /// press doesn't also count as one outside the menu.
    pressing_button: bool,
}

impl AccountMenu {
    pub fn is_open(&self) -> bool {
        self.focus.is_some()
    }
}

/// What an item does.
#[derive(Clone, Copy)]
enum Item {
    ManagePlan,
    Upgrade,
    AccountSettings,
    OpenWeb,
    SignOut,
}

impl SidebarView {
    pub(super) fn toggle_account_menu(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.account_menu.pressing_button = false;
        if self.account_menu.is_open() {
            self.close_account_menu(cx);
            return;
        }
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        self.account_menu.focus = Some(focus);
        self.account_menu.opened += 1;
        cx.notify();
    }

    /// Notes a press on the account button before the menu sees it as one outside.
    pub(super) fn press_account_button(&mut self) {
        self.account_menu.pressing_button = self.account_menu.is_open();
    }

    fn close_account_menu(&mut self, cx: &mut Context<'_, Self>) {
        if self.account_menu.focus.take().is_some() {
            cx.emit(SidebarEvent::ReturnFocus);
            cx.notify();
        }
    }

    fn press_outside_account_menu(&mut self, cx: &mut Context<'_, Self>) {
        if !self.account_menu.pressing_button {
            self.close_account_menu(cx);
        }
    }

    pub(super) fn account_menu(&mut self, cx: &mut Context<'_, Self>) -> Option<AnyElement> {
        let focus = self.account_menu.focus.clone()?;
        let who = self.identity.who(cx);
        let theme = Theme::get(cx).clone();
        let body = match &who {
            Who::ThisMac => self.signed_out_items(cx),
            Who::Account { name, plan, .. } => self.signed_in_items(&who, name, *plan, cx),
        };
        let panel = div()
            .id("account-menu")
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .on_action(cx.listener(|this, _: &CloseAccountMenu, _, cx| this.close_account_menu(cx)))
            .on_mouse_down_out(cx.listener(|this, _, _, cx| this.press_outside_account_menu(cx)))
            .occlude()
            .absolute()
            .left(INSET)
            .w(WIDTH)
            .p(px(5.))
            .flex()
            .flex_col()
            .rounded(px(12.))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .shadow_lg()
            .text_size(px(13.))
            .text_color(theme.text)
            .children(body);
        let bottom = FOOTER_HEIGHT + ABOVE_FOOTER;
        let open = Animation::new(motion::MENU_OPEN);
        Some(
            panel
                .with_animation(
                    ("account-menu-open", self.account_menu.opened),
                    open,
                    move |panel, delta| {
                        panel
                            .opacity(delta)
                            .bottom(bottom - px(motion::MENU_SLIDE * (1. - delta)))
                    },
                )
                .into_any_element(),
        )
    }

    fn signed_in_items(
        &self,
        who: &Who,
        name: &SharedString,
        plan: Option<Plan>,
        cx: &mut Context<'_, Self>,
    ) -> Vec<AnyElement> {
        let theme = Theme::get(cx).clone();
        let plan_line = match plan {
            Some(Plan::Pro) => "Knoa Pro",
            Some(Plan::Free) => "Free plan",
            None => "Signed in",
        };
        let mut items = vec![
            header(
                who,
                name.clone(),
                SharedString::new_static(plan_line),
                &theme,
            ),
            separator(&theme),
            self.sync_line(cx),
            separator(&theme),
        ];
        items.extend(match plan {
            Some(Plan::Pro) => Some(item(
                "Manage plan…",
                line_icon!("credit-card"),
                Item::ManagePlan,
                false,
                cx,
            )),
            Some(Plan::Free) => Some(item(
                "Upgrade to Pro",
                line_icon!("zap"),
                Item::Upgrade,
                true,
                cx,
            )),
            None => None,
        });
        items.push(item(
            "Account settings…",
            line_icon!("circle-user-round"),
            Item::AccountSettings,
            false,
            cx,
        ));
        items.push(item(
            "Open Knoa on the web",
            line_icon!("external-link"),
            Item::OpenWeb,
            false,
            cx,
        ));
        items.push(separator(&theme));
        items.push(item(
            "Sign out",
            line_icon!("log-out"),
            Item::SignOut,
            false,
            cx,
        ));
        items
    }

    /// How sync is doing, as a line rather than a button.
    fn sync_line(&self, cx: &mut Context<'_, Self>) -> AnyElement {
        let theme = Theme::get(cx);
        let cloud = self.cloud.read(cx);
        let (icon, text, color, retry) = if matches!(cloud.phase(), Phase::OtherAccount(_)) {
            (
                line_icon!("cloud-alert"),
                "Sync paused".into(),
                theme.danger,
                false,
            )
        } else {
            match cloud.sync() {
                SyncStatus::Error(_) => (
                    line_icon!("cloud-alert"),
                    "Sync stopped".into(),
                    theme.danger,
                    true,
                ),
                SyncStatus::Offline => (
                    line_icon!("cloud-off"),
                    offline(cloud.waiting_changes()),
                    theme.muted,
                    false,
                ),
                SyncStatus::Starting | SyncStatus::Syncing(_) | SyncStatus::UpToDate => {
                    match cloud.last_synced() {
                        Some(at) => (
                            line_icon!("cloud-check"),
                            synced(at, Timestamp::now()),
                            theme.muted,
                            false,
                        ),
                        None => (
                            line_icon!("refresh-cw"),
                            "Syncing…".into(),
                            theme.muted,
                            false,
                        ),
                    }
                }
            }
        };
        div()
            .flex()
            .items_center()
            .gap(px(9.))
            .h(ITEM_HEIGHT)
            .px(px(8.))
            .text_size(px(12.5))
            .text_color(color)
            .child(svg_icon(icon, ITEM_ICON, color))
            .child(div().flex_1().min_w_0().truncate().child(text))
            .when(retry, |s| {
                s.child(
                    text_button("Retry", theme).on_click(cx.listener(|this, _, _, cx| {
                        this.cloud.update(cx, Cloud::retry_sync);
                    })),
                )
            })
            .into_any_element()
    }

    fn signed_out_items(&self, cx: &mut Context<'_, Self>) -> Vec<AnyElement> {
        let theme = Theme::get(cx);
        let cloud = self.cloud.read(cx);
        let available = cloud.config().is_some();
        let waiting = matches!(cloud.phase(), Phase::SigningIn(_));
        let mut items = vec![header(
            &Who::ThisMac,
            SharedString::new_static("This Mac"),
            SharedString::new_static("Not signed in"),
            theme,
        )];
        if !available {
            items.push(note("Sync isn't available in this build.", theme));
            return items;
        }
        items.push(note(SIGNED_OUT_NOTE, theme));
        items.push(if waiting {
            self.finish_in_browser(cx)
        } else {
            self.sign_in_button(cx)
        });
        items
    }

    fn sign_in_button(&self, cx: &mut Context<'_, Self>) -> AnyElement {
        let theme = Theme::get(cx);
        let hover = theme.accent.opacity(0.88);
        div()
            .id("sign-in")
            .flex()
            .items_center()
            .justify_center()
            .h(px(32.))
            .mx(px(3.))
            .mb(px(4.))
            .rounded(px(8.))
            .bg(theme.accent)
            .text_color(theme.on_accent)
            .font_weight(FontWeight::SEMIBOLD)
            .cursor_pointer()
            .hover(move |s| s.bg(hover))
            .on_click(cx.listener(|this, _, _, cx| this.cloud.update(cx, Cloud::sign_in)))
            .child("Sign in…")
            .into_any_element()
    }

    /// While the browser has the sign-in: a turning circle, and a way to give up.
    fn finish_in_browser(&self, cx: &mut Context<'_, Self>) -> AnyElement {
        let theme = Theme::get(cx);
        let turn = Animation::new(motion::SLOW_TURN / 4)
            .repeat()
            .with_max_fps(motion::LOOP_FPS);
        let circle = svg_icon(line_icon!("circle-dashed"), ITEM_ICON, theme.muted).with_animation(
            "finish-in-browser",
            turn,
            |svg, delta| {
                svg.with_transformation(gpui::Transformation::rotate(gpui::Radians(
                    delta * std::f32::consts::TAU,
                )))
            },
        );
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(32.))
            .mx(px(3.))
            .mb(px(4.))
            .pl(px(9.))
            .pr(px(4.))
            .rounded(px(8.))
            .bg(theme.soft)
            .text_size(px(12.5))
            .text_color(theme.muted)
            .child(circle)
            .child(div().flex_1().child("Finish in browser"))
            .child(
                text_button("Cancel", theme).on_click(cx.listener(|this, _, _, cx| {
                    this.cloud.update(cx, Cloud::cancel_sign_in);
                })),
            )
            .into_any_element()
    }

    fn choose(&mut self, item: Item, cx: &mut Context<'_, Self>) {
        self.close_account_menu(cx);
        let config = self.cloud.read(cx).config().cloned();
        match item {
            Item::ManagePlan | Item::Upgrade | Item::OpenWeb => {
                let Some(config) = config else { return };
                let url = match item {
                    Item::ManagePlan => config.billing,
                    Item::Upgrade => config.pricing,
                    _ => config.web,
                };
                cx.open_url(url.as_str());
            }
            Item::AccountSettings => cx.emit(SidebarEvent::OpenSettings(Page::Account)),
            Item::SignOut => self.cloud.update(cx, Cloud::sign_out),
        }
    }
}

fn header(who: &Who, name: SharedString, sub: SharedString, theme: &Theme) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .pt(px(8.))
        .px(px(8.))
        .pb(px(7.))
        .child(avatar(who, MENU_AVATAR, theme))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(1.))
                .child(
                    div()
                        .truncate()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(name),
                )
                .child(
                    div()
                        .truncate()
                        .text_size(px(11.5))
                        .text_color(theme.muted)
                        .child(sub),
                ),
        )
        .into_any_element()
}

fn separator(theme: &Theme) -> AnyElement {
    div()
        .h(px(1.))
        .my(px(4.))
        .mx(px(6.))
        .bg(theme.hairline)
        .into_any_element()
}

fn note(text: &'static str, theme: &Theme) -> AnyElement {
    div()
        .pt(px(2.))
        .px(px(8.))
        .pb(px(8.))
        .text_size(px(12.))
        .line_height(gpui::relative(1.45))
        .text_color(theme.muted)
        .child(text)
        .into_any_element()
}

fn item(
    label: &'static str,
    icon: SharedString,
    action: Item,
    accent: bool,
    cx: &mut Context<'_, SidebarView>,
) -> AnyElement {
    let theme = Theme::get(cx);
    let (text, icon_color) = if accent {
        (theme.accent, theme.accent)
    } else {
        (theme.text, theme.muted)
    };
    let hover = theme.hover;
    div()
        .id(label)
        .flex()
        .items_center()
        .gap(px(9.))
        .h(ITEM_HEIGHT)
        .px(px(8.))
        .rounded(px(7.))
        .text_color(text)
        .when(accent, |s| s.font_weight(FontWeight::SEMIBOLD))
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .on_click(cx.listener(move |this, _, _, cx| this.choose(action, cx)))
        .child(svg_icon(icon, ITEM_ICON, icon_color))
        .child(label)
        .into_any_element()
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
