//! The settings sidebar: the app-wide pages, then every workspace and "New workspace".

use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _,
    px,
};

use super::{Page, SettingsView};
use crate::ui::{
    components::{icon, workspace_mark},
    theme::{PANEL_RADIUS, SERIF, Theme},
};

const NAV_WIDTH: gpui::Pixels = px(220.);

impl SettingsView {
    pub(super) fn nav(&self, cx: &mut Context<'_, SettingsView>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let workspaces = self.library.read(cx).workspaces().to_vec();

        let mut app_pages = div().flex().flex_col().gap_0p5().px_2();
        for (page, label, icon_name) in [
            (Page::General, "General", "settings-2"),
            (Page::Appearance, "Appearance", "palette"),
            (Page::Ai, "AI", "sparkle"),
        ] {
            let mark = icon(icon_name, px(15.), self.icon_color(page, &theme));
            app_pages = app_pages.child(self.nav_row(page, label, mark, cx));
        }

        let mut workspace_pages = div().flex().flex_col().gap_0p5().px_2();
        for workspace in &workspaces {
            let mark = workspace_mark(&workspace.name, workspace.color, px(16.), cx);
            let page = Page::Workspace(workspace.id);
            workspace_pages = workspace_pages.child(self.nav_row(page, &workspace.name, mark, cx));
        }
        let plus = icon("plus", px(15.), theme.muted);
        workspace_pages =
            workspace_pages.child(self.nav_row(Page::NewWorkspace, "New workspace", plus, cx));

        // One radius less than the sheet's, so the fill sits inside its 1 px border.
        div()
            .id("settings-nav")
            .w(NAV_WIDTH)
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .rounded_l(PANEL_RADIUS - px(1.))
            .bg(theme.soft)
            .border_r_1()
            .border_color(theme.hairline)
            .child(
                div()
                    .px_4()
                    .pt_10()
                    .pb_4()
                    .font_family(SERIF)
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Settings"),
            )
            .child(app_pages)
            .child(
                div()
                    .px_4()
                    .pt_5()
                    .pb_1()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.faint)
                    .child("Workspaces"),
            )
            .child(workspace_pages)
    }

    fn icon_color(&self, page: Page, theme: &Theme) -> gpui::Hsla {
        if self.page == page {
            theme.accent
        } else {
            theme.muted
        }
    }

    /// Looks like a place in the main sidebar, so the two read as one app.
    fn nav_row(
        &self,
        page: Page,
        label: &str,
        mark: impl IntoElement,
        cx: &mut Context<'_, SettingsView>,
    ) -> AnyElement {
        let theme = Theme::get(cx);
        let selected = self.page == page;
        let muted = page == Page::NewWorkspace && !selected;
        let id = SharedString::from(format!("settings-nav-{page:?}"));
        div()
            .id(id)
            .flex()
            .items_center()
            .gap_2()
            .h(px(30.))
            .px_2()
            .rounded(px(9.))
            .text_sm()
            .cursor_pointer()
            .when(muted, |s| s.text_color(theme.muted))
            .when(selected, |s| {
                s.bg(theme.hover).font_weight(FontWeight::MEDIUM)
            })
            .hover(|s| s.bg(theme.hover))
            .child(mark)
            .child(div().flex_1().min_w_0().truncate().child(label.to_owned()))
            .on_click(cx.listener(move |this, _, window, cx| {
                if this.page != page {
                    this.show(page, window, cx);
                }
            }))
            .into_any_element()
    }
}
