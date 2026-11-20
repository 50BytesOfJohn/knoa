//! The workspace page's header and its name and color section.

use gpui::{
    Context, Div, InteractiveElement as _, IntoElement as _, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, px,
};
use gpui_component::input::Input;
use knoa_core::{
    Ink,
    protocol::max_chars::{self, fits},
};

use super::{WorkspacePage, browsers::synced_from, form};
use crate::{
    i18n::{self, t},
    ui::{
        components::{Tone, button, grouped, tip, workspace_mark},
        theme::Theme,
    },
};

impl WorkspacePage {
    pub(super) fn header(&self, theme: &Theme, cx: &mut Context<'_, WorkspacePage>) -> Div {
        let Some(id) = self.existing else {
            return form::header(
                t!("settings-workspace-new"),
                &t!("settings-workspace-new-about"),
                theme,
            );
        };
        let typed = self.name.read(cx).value().trim().to_owned();
        let title = if typed.is_empty() {
            self.stored(id, cx).map(|w| w.name).unwrap_or_default()
        } else {
            typed
        };
        let is_current = self.library.read(cx).workspace_id() == id;
        let switch = if is_current {
            div()
                .text_sm()
                .text_color(theme.muted)
                .child(t!("settings-workspace-shown-now"))
                .into_any_element()
        } else {
            button(
                "show-workspace",
                t!("settings-workspace-show-it"),
                Tone::Soft,
                cx,
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.save(cx);
                this.library
                    .update(cx, |library, cx| library.set_workspace(id, cx));
            }))
            .into_any_element()
        };
        div()
            .flex()
            .items_start()
            .gap_4()
            .child(workspace_mark(&title, self.color, px(40.), cx))
            .child(div().flex_1().min_w_0().child(form::header(
                title,
                &synced_from(&self.browsers),
                theme,
            )))
            .child(div().pt_1().child(switch))
    }

    pub(super) fn identity_group(&self, theme: &Theme, cx: &mut Context<'_, WorkspacePage>) -> Div {
        let mut swatches = div().flex().gap_2();
        for ink in Ink::ALL {
            let selected = ink == self.color;
            swatches = swatches.child(
                div()
                    .id(("workspace-color", ink.index()))
                    .size(px(22.))
                    .rounded(px(7.))
                    .border_2()
                    .border_color(if selected {
                        theme.text
                    } else {
                        gpui::transparent_black()
                    })
                    .p(px(2.))
                    .cursor_pointer()
                    .child(div().size_full().rounded(px(4.)).bg(theme.ink(ink)))
                    .tooltip(tip(i18n::ink_label(ink), None, theme))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.color = ink;
                        this.changed(cx);
                    })),
            );
        }
        let mut rows = vec![form::row(
            &t!("settings-workspace-name"),
            None,
            div().w(px(240.)).child(Input::new(&self.name)),
            theme,
        )];
        let name = self.name.read(cx).value();
        let max = max_chars::workspace::NAME;
        if !fits(name.trim(), max) {
            let reason = t!(
                "settings-workspace-name-too-long",
                max = (grouped(max as u64))
            );
            rows.push(form::problem_note(reason, theme));
        }
        rows.push(form::row(
            &t!("settings-workspace-color"),
            None,
            swatches,
            theme,
        ));
        form::group(&t!("settings-workspace-name-and-color"), rows, theme)
    }
}
