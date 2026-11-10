//! The workspace page's organizing section: instructions for the AI, screening by a
//! guideline, and organizing the workspace again.

use gpui::{Context, Div, IntoElement as _, px};
use gpui_component::{input::Textarea, switch::Switch};

use super::{WorkspacePage, form};
use crate::{
    services::organizer::{AiStatus, Organizer},
    ui::theme::Theme,
};

impl WorkspacePage {
    pub(super) fn organizing_group(
        &self,
        theme: &Theme,
        cx: &mut Context<'_, WorkspacePage>,
    ) -> Div {
        let mut rows = vec![
            form::stacked(
                "Instructions",
                Some("How the AI should sort, tag and summarize what lands here."),
                Textarea::new(&self.instructions).h(px(84.)),
                theme,
            ),
            form::row(
                "Only keep bookmarks that fit",
                Some(
                    "Off: everything from the browser lands here. On: the AI sets aside what doesn't fit, and you can keep it anyway.",
                ),
                Switch::new("screening")
                    .checked(self.screening)
                    .on_click(cx.listener(|this, on: &bool, _, cx| {
                        this.screening = *on;
                        this.changed(cx);
                    })),
                theme,
            ),
        ];
        if self.screening {
            rows.push(form::stacked(
                "What belongs here",
                None,
                Textarea::new(&self.guideline).h(px(84.)),
                theme,
            ));
        }
        let ai_on = *Organizer::entity(cx).read(cx).status() != AiStatus::Off;
        if let Some(reorganize) = self.reorganize.clone().filter(|_| ai_on) {
            rows.push(reorganize.into_any_element());
        }
        form::group("Organizing", rows, theme)
    }
}
