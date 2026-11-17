//! What the account menu shows: signed in, the account, its sync, its plan and where to
//! go next; signed out, a single item to sign in.

use gpui::{
    Animation, AnimationExt as _, AnyElement, Context, Div, FontWeight, InteractiveElement as _,
    IntoElement, ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
    div, px,
};
use knoa_core::protocol::Plan;

use super::{ITEM_HEIGHT, ITEM_ICON, Item};
use crate::{
    assets::line_icon,
    services::cloud::{Cloud, Phase},
    ui::{
        components::svg_icon,
        main::sidebar::{
            SidebarView,
            account::{MENU_AVATAR, Who, avatar},
            status_card::text_button,
        },
        motion,
        theme::Theme,
    },
};

impl SidebarView {
    pub(super) fn account_menu_items(&mut self, cx: &mut Context<'_, Self>) -> Vec<AnyElement> {
        let who = self.identity.who(cx);
        match &who {
            Who::ThisMac => vec![self.sign_in_item(cx)],
            Who::Account { name, plan, .. } => self.signed_in_items(&who, name.clone(), *plan, cx),
        }
    }

    fn signed_in_items(
        &mut self,
        who: &Who,
        name: SharedString,
        plan: Option<Plan>,
        cx: &mut Context<'_, Self>,
    ) -> Vec<AnyElement> {
        let theme = Theme::get(cx).clone();
        let mut items = vec![
            header(who, name, plan_line(plan), &theme),
            separator(&theme),
            self.sync_line(cx),
            separator(&theme),
        ];
        items.extend(plan_item(plan, cx));
        items.push(item(
            "Account settings…",
            line_icon!("circle-user-round"),
            Item::AccountSettings,
            cx,
        ));
        items.push(item(
            "Open Knoa on the web",
            line_icon!("external-link"),
            Item::OpenWeb,
            cx,
        ));
        items.push(separator(&theme));
        items.push(item("Sign out", line_icon!("log-out"), Item::SignOut, cx));
        items
    }

    /// "Sign in…", or while the browser has the sign-in, a way to wait for it or give up.
    fn sign_in_item(&self, cx: &mut Context<'_, Self>) -> AnyElement {
        if matches!(self.cloud.read(cx).phase(), Phase::SigningIn(_)) {
            self.finish_in_browser(cx)
        } else {
            item("Sign in…", line_icon!("log-in"), Item::SignIn, cx)
        }
    }

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
        item_row()
            .pr(px(2.))
            .text_color(theme.muted)
            .child(circle)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child("Finish in browser"),
            )
            .child(
                text_button("Cancel", theme).on_click(cx.listener(|this, _, _, cx| {
                    this.cloud.update(cx, Cloud::cancel_sign_in);
                })),
            )
            .into_any_element()
    }
}

fn plan_line(plan: Option<Plan>) -> SharedString {
    SharedString::new_static(match plan {
        Some(Plan::Pro) => "Knoa Pro",
        Some(Plan::Free) => "Free plan",
        None => "Signed in",
    })
}

fn plan_item(plan: Option<Plan>, cx: &mut Context<'_, SidebarView>) -> Option<AnyElement> {
    match plan? {
        Plan::Pro => Some(item(
            "Manage plan…",
            line_icon!("credit-card"),
            Item::ManagePlan,
            cx,
        )),
        Plan::Free => Some(item("Upgrade to Pro", line_icon!("zap"), Item::Upgrade, cx)),
    }
}

fn header(who: &Who, name: SharedString, sub: SharedString, theme: &Theme) -> AnyElement {
    let lines = div()
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
        );
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .pt(px(8.))
        .px(px(8.))
        .pb(px(7.))
        .child(avatar(who, MENU_AVATAR, theme))
        .child(lines)
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

/// One row of the menu, the same for every item: an icon, then its label.
fn item(
    label: &'static str,
    icon: SharedString,
    action: Item,
    cx: &mut Context<'_, SidebarView>,
) -> AnyElement {
    let theme = Theme::get(cx);
    let hover = theme.hover;
    item_row()
        .id(label)
        .text_color(theme.text)
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .on_click(cx.listener(move |this, _, _, cx| this.choose(action, cx)))
        .child(svg_icon(icon, ITEM_ICON, theme.muted))
        .child(label)
        .into_any_element()
}

fn item_row() -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(9.))
        .h(ITEM_HEIGHT)
        .px(px(8.))
        .rounded(px(7.))
}
