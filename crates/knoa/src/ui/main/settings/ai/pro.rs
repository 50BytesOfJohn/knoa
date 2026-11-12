//! The AI page's Pro card: what Pro does instead of a key of your own, and the way to it
//! from where the account stands.

use gpui::{
    App, Div, FontWeight, ParentElement as _, StatefulInteractiveElement as _, Styled as _, div, px,
};
use knoa_core::protocol::Plan;

use crate::{
    services::cloud::{Cloud, Phase},
    ui::{
        components::{Tone, button, icon},
        theme::{CARD_RADIUS, Theme},
    },
};

const PITCH: &str = "Knoa organizes your bookmarks in the cloud. No key to set up, no bill per request, and your library syncs across your devices.";

/// `None` when Pro isn't on offer: a build without a cloud, an account that has it, or
/// one still being looked up or waiting on a choice on the Account page.
pub fn card(theme: &Theme, cx: &App) -> Option<Div> {
    let cloud = Cloud::entity(cx);
    let cloud = cloud.read(cx);
    let pricing = cloud.config()?.pricing.clone();
    let signed_out = match cloud.phase() {
        Phase::SignedOut | Phase::SigningIn(_) => true,
        Phase::SignedIn(_) if cloud.account()?.plan == Plan::Free => false,
        Phase::SignedIn(_) | Phase::OtherAccount(_) => return None,
    };
    let label = if signed_out { "Get Pro" } else { "Upgrade" };
    let mut actions = div().flex().items_center().gap_2().child(
        button("get-pro", label, Tone::Primary, cx)
            .on_click(move |_, _, cx| cx.open_url(pricing.as_str())),
    );
    if signed_out && matches!(cloud.phase(), Phase::SignedOut) {
        // Someone who already pays only needs to sign in on this Mac.
        actions = actions.child(
            button("pro-sign-in", "I have Pro", Tone::Ghost, cx)
                .on_click(|_, _, cx| Cloud::entity(cx).update(cx, Cloud::sign_in)),
        );
    }
    Some(
        div()
            .flex()
            .items_start()
            .gap_3()
            .p_5()
            .rounded(CARD_RADIUS)
            .bg(theme.ai_soft)
            .border_1()
            .border_color(theme.ai.opacity(0.35))
            .child(icon("sparkles", px(18.), theme.ai).mt(px(2.)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(text(theme))
                    .child(actions),
            ),
    )
}

fn text(theme: &Theme) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.ai_text)
                .child("Let Knoa do it with Pro"),
        )
        .child(div().text_sm().text_color(theme.text).child(PITCH))
}
