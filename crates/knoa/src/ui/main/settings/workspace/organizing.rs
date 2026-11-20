//! The workspace page's organizing section: instructions for the AI, screening by a
//! guideline, and organizing the workspace again.

use gpui::{Context, Div, IntoElement as _, px};
use gpui_component::{input::Textarea, switch::Switch};
use knoa_core::protocol::max_chars::{
    fits,
    workspace::{GUIDELINE, INSTRUCTIONS},
};

use super::{WorkspacePage, form};
use crate::{
    i18n::t,
    services::organizer::{AiStatus, Organizer},
    ui::theme::Theme,
};

impl WorkspacePage {
    pub(super) fn organizing_group(
        &self,
        theme: &Theme,
        cx: &mut Context<'_, WorkspacePage>,
    ) -> Div {
        let mut rows = vec![form::stacked(
            &t!("settings-workspace-instructions"),
            Some(&t!("settings-workspace-instructions-about")),
            Textarea::new(&self.instructions).h(px(84.)),
            theme,
        )];
        let instructions = self.instructions.read(cx).value();
        if !fits(instructions.trim(), INSTRUCTIONS) {
            let reason = t!(
                "settings-workspace-instructions-too-long",
                max = INSTRUCTIONS
            );
            rows.push(form::problem_note(reason, theme));
        }
        rows.push(form::row(
            &t!("settings-workspace-screening"),
            Some(&t!("settings-workspace-screening-about")),
            Switch::new("screening")
                .checked(self.screening)
                .on_click(cx.listener(|this, on: &bool, _, cx| {
                    this.screening = *on;
                    this.changed(cx);
                })),
            theme,
        ));
        if self.screening {
            rows.push(form::stacked(
                &t!("settings-workspace-guideline"),
                None,
                Textarea::new(&self.guideline).h(px(84.)),
                theme,
            ));
            let guideline = self.guideline.read(cx).value();
            if !fits(guideline.trim(), GUIDELINE) {
                let reason = t!("settings-workspace-guideline-too-long", max = GUIDELINE);
                rows.push(form::problem_note(reason, theme));
            }
        }
        let ai_on = *Organizer::entity(cx).read(cx).status() != AiStatus::Off;
        if let Some(reorganize) = self.reorganize.clone().filter(|_| ai_on) {
            rows.push(reorganize.into_any_element());
        }
        form::group(&t!("settings-workspace-organizing"), rows, theme)
    }
}
