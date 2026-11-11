//! The folder editor's colors: the theme's inks, and a slider for a hue of your own.

use gpui::{
    App, AppContext as _, Context, Div, Entity, Hsla, InteractiveElement as _, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Subscription, div, prelude::FluentBuilder as _,
    px,
};
use gpui_component::slider::{Slider, SliderEvent, SliderState};
use knoa_core::{FolderColor, Hue, Ink};

use super::FolderEditor;
use crate::ui::{components::tip, theme::Theme};

const SWATCH: f32 = 22.;
/// Enough steps that the track reads as a smooth band; GPUI gradients take only two stops.
const TRACK_STEPS: u16 = 60;

/// The slider for a custom hue, starting at the folder's hue, or at its ink's so moving
/// it from there starts close to the color already picked.
pub(super) fn hue_state(color: FolderColor, cx: &mut App) -> Entity<SliderState> {
    let degrees = match color {
        FolderColor::Hue(hue) => hue.degrees(),
        FolderColor::Ink(ink) => Hue::clamped(Theme::get(cx).ink(ink).h * 360.).degrees(),
    };
    cx.new(|_| {
        SliderState::new()
            .min(f32::from(Hue::MIN))
            .max(f32::from(Hue::MAX))
            .step(1.)
            .default_value(f32::from(degrees))
    })
}

/// Moving the slider picks its hue.
pub(super) fn follow_hue(
    hue: &Entity<SliderState>,
    cx: &mut Context<'_, FolderEditor>,
) -> Subscription {
    cx.subscribe(hue, |this, _, event: &SliderEvent, cx| {
        if let SliderEvent::Change(value) = event {
            this.color = FolderColor::Hue(Hue::clamped(value.end()));
            cx.notify();
        }
    })
}

impl FolderEditor {
    pub(super) fn color_picker(&self, theme: &Theme, cx: &mut Context<'_, FolderEditor>) -> Div {
        let mut inks = div().flex().gap(px(6.));
        for ink in Ink::ALL {
            let selected = self.color == FolderColor::Ink(ink);
            inks = inks.child(
                swatch(("ink", ink.index()), theme.ink(ink), selected, theme)
                    .tooltip(tip(ink.label(), None, theme))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.color = FolderColor::Ink(ink);
                        cx.notify();
                    })),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(inks)
            .child(self.hue_row(theme, cx))
    }

    /// A swatch of the slider's hue, selected while it is the folder's color, beside the
    /// slider itself.
    fn hue_row(&self, theme: &Theme, cx: &mut Context<'_, FolderEditor>) -> Div {
        let hue = Hue::clamped(self.hue.read(cx).value().end());
        let selected = matches!(self.color, FolderColor::Hue(_));
        div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                swatch("custom-hue", theme.hue(hue), selected, theme)
                    .tooltip(tip("Custom", None, theme))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.color = FolderColor::Hue(hue);
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .h_8()
                    .flex()
                    .items_center()
                    .child(hue_track(theme))
                    .child(
                        Slider::new(&self.hue)
                            .bg(gpui::transparent_black())
                            .text_color(theme.text),
                    ),
            )
    }
}

fn swatch(
    id: impl Into<gpui::ElementId>,
    color: Hsla,
    selected: bool,
    theme: &Theme,
) -> gpui::Stateful<Div> {
    div()
        .id(id)
        .size(px(SWATCH))
        .flex_none()
        .rounded_full()
        .bg(color)
        .cursor_pointer()
        .when(selected, |s| s.border_2().border_color(theme.text))
}

/// The theme's color at each step of the slider's range, drawn under the slider the way
/// gpui-component's own color picker draws its hue track.
fn hue_track(theme: &Theme) -> Div {
    let span = f32::from(Hue::MAX - Hue::MIN);
    let steps = (0..TRACK_STEPS).map(|step| {
        let degrees = f32::from(Hue::MIN) + span * f32::from(step) / f32::from(TRACK_STEPS - 1);
        div().flex_1().h_full().bg(theme.hue(Hue::clamped(degrees)))
    });
    div()
        .absolute()
        .left_0()
        .right_0()
        .h_2p5()
        .flex()
        .rounded_full()
        .overflow_hidden()
        .children(steps)
}
