//! The account menu, opening upward from the footer's account button: when it is open,
//! how it closes, and what its items do. What it shows is in `content` and `sync_line`.
//!
//! It closes on Esc, on a press outside it, when focus moves elsewhere and after an item
//! is chosen.

mod content;
mod sync_line;

use std::mem;

use gpui::{
    Animation, AnimationExt as _, AnyElement, App, Context, FocusHandle, InteractiveElement as _,
    IntoElement, KeyBinding, ParentElement as _, Pixels, Styled as _, Subscription, Window,
    actions, div, px,
};

use self::sync_line::SyncedAge;
use super::{INSET, SidebarEvent, SidebarView, footer::FOOTER_HEIGHT};
use crate::{
    services::cloud::Cloud,
    ui::{main::settings::Page, motion, theme::Theme},
};

actions!(account_menu, [CloseAccountMenu]);

const KEY_CONTEXT: &str = "AccountMenu";
const WIDTH: Pixels = px(224.);
/// From the footer's top line to the menu's bottom edge.
const ABOVE_FOOTER: Pixels = px(4.);
const ITEM_HEIGHT: Pixels = px(30.);
const ITEM_ICON: Pixels = px(15.);

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
    open: Option<Open>,
    /// Goes up with each opening, which replays the opening motion.
    opened: usize,
    /// A press on the account button is under way. It toggles the menu itself, so the
    /// press doesn't also count as one outside the menu.
    pressing_button: bool,
}

/// What lives only while the menu is open, and goes with it.
struct Open {
    /// Held so Esc reaches the menu.
    focus: FocusHandle,
    _focus_out: Subscription,
    age: SyncedAge,
}

impl AccountMenu {
    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }
}

/// What an item does.
#[derive(Clone, Copy)]
enum Item {
    SignIn,
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
        // Focus moved on by a shortcut or a click elsewhere: the menu goes, and leaves
        // focus where it went.
        let focus_out = cx.on_focus_out(&focus, window, |this, _, _, cx| {
            if this.account_menu.open.take().is_some() {
                cx.notify();
            }
        });
        self.account_menu.open = Some(Open {
            focus,
            _focus_out: focus_out,
            age: SyncedAge::default(),
        });
        self.account_menu.opened += 1;
        cx.notify();
    }

    /// Notes a press on the account button before the menu sees it as one outside.
    pub(super) fn press_account_button(&mut self) {
        self.account_menu.pressing_button = self.account_menu.is_open();
    }

    fn close_account_menu(&mut self, cx: &mut Context<'_, Self>) {
        if self.account_menu.open.take().is_some() {
            cx.emit(SidebarEvent::ReturnFocus);
            cx.notify();
        }
    }

    fn press_outside_account_menu(&mut self, cx: &mut Context<'_, Self>) {
        // Only for the press that set it: a press that never became a click must not
        // keep later outside presses from closing the menu.
        if !mem::take(&mut self.account_menu.pressing_button) {
            self.close_account_menu(cx);
        }
    }

    pub(super) fn account_menu(&mut self, cx: &mut Context<'_, Self>) -> Option<AnyElement> {
        let focus = self.account_menu.open.as_ref()?.focus.clone();
        let body = self.account_menu_items(cx);
        let theme = Theme::get(cx);
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

    fn choose(&mut self, item: Item, cx: &mut Context<'_, Self>) {
        let config = self.cloud.read(cx).config().cloned();
        if let (Item::SignIn, Some(_)) = (item, &config) {
            // The menu stays open, so the item can say to finish in the browser.
            self.cloud.update(cx, Cloud::sign_in);
            return;
        }
        self.close_account_menu(cx);
        match item {
            // Without a cloud in this build, Account settings says so.
            Item::SignIn | Item::AccountSettings => {
                cx.emit(SidebarEvent::OpenSettings(Page::Account));
            }
            Item::ManagePlan | Item::Upgrade | Item::OpenWeb => {
                let Some(config) = config else { return };
                let url = match item {
                    Item::ManagePlan => config.billing,
                    Item::Upgrade => config.pricing,
                    _ => config.web,
                };
                cx.open_url(url.as_str());
            }
            Item::SignOut => self.cloud.update(cx, Cloud::sign_out),
        }
    }
}
