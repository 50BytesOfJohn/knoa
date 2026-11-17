//! The sidebar's footer: the account button, which opens the account menu, and settings.

use gpui::{
    Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};

use super::{
    INSET, SidebarEvent, SidebarView,
    account::{FOOTER_AVATAR, avatar},
    titlebar::{BUTTON, icon_button},
};
use crate::{
    assets::line_icon,
    ui::{
        components::{svg_icon, tip},
        main::{OpenSettings, settings::Page},
        theme::Theme,
    },
};

pub(super) const FOOTER_HEIGHT: Pixels = px(46.);
const ACCOUNT_BUTTON: Pixels = px(32.);

impl SidebarView {
    pub(super) fn footer(&mut self, cx: &mut Context<'_, SidebarView>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let settings = icon_button("settings", line_icon!("settings"), BUTTON, px(16.), theme)
            .tooltip(tip("Settings", Some(&OpenSettings), theme))
            .on_click(cx.listener(|_, _, _, cx| {
                cx.emit(SidebarEvent::OpenSettings(Page::General));
            }));
        div()
            .flex_none()
            .h(FOOTER_HEIGHT)
            .px(INSET)
            .flex()
            .items_center()
            .gap(px(2.))
            .border_t_1()
            .border_color(theme.hairline)
            .child(self.account_button(cx))
            .child(settings)
    }

    fn account_button(&mut self, cx: &mut Context<'_, SidebarView>) -> impl IntoElement {
        let who = self.identity.who(cx);
        let open = self.account_menu.is_open();
        let theme = Theme::get(cx);
        div()
            .id("account-button")
            .flex_1()
            .min_w_0()
            .h(ACCOUNT_BUTTON)
            .pl(px(5.))
            .pr(px(8.))
            .flex()
            .items_center()
            .gap(px(8.))
            .rounded(px(10.))
            .cursor_pointer()
            .when(open, |s| s.bg(theme.selected_row()))
            .when(!open, |s| s.hover(|s| s.bg(theme.hover)))
            .capture_any_mouse_down(cx.listener(|this, _, _, _| this.press_account_button()))
            .on_click(cx.listener(|this, _, window, cx| this.toggle_account_menu(window, cx)))
            .child(avatar(&who, FOOTER_AVATAR, theme))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(who.name()),
            )
            .child(svg_icon(
                line_icon!("chevrons-up-down"),
                px(12.),
                theme.muted,
            ))
    }
}
