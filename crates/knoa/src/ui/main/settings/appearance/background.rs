//! The Background group: what the main window shows behind its panels, and for the style
//! chosen, its effect or picture and how strongly it shows.

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, IntoElement, ObjectFit, ParentElement as _,
    Render, StatefulInteractiveElement as _, Styled as _, StyledImage as _, Subscription, Window,
    div, img, px,
};
use gpui_component::slider::{Slider, SliderEvent, SliderState};

use crate::{
    i18n::t,
    state::settings::{Background, BackgroundStyle, Effect, Settings, SettingsFile},
    ui::{
        backdrop::{self, BackdropImage, PictureImport},
        choice,
        components::{Tone, button},
        main::settings::form,
        theme::{ROW_RADIUS, Theme},
    },
};

const THUMBNAIL_WIDTH: gpui::Pixels = px(48.);
const THUMBNAIL_HEIGHT: gpui::Pixels = px(32.);
const SLIDER_WIDTH: gpui::Pixels = px(160.);

pub struct BackgroundGroup {
    opacity: Entity<SliderState>,
    image_opacity: Entity<SliderState>,
    _subscriptions: Vec<Subscription>,
}

impl BackgroundGroup {
    pub fn new(cx: &mut Context<'_, BackgroundGroup>) -> BackgroundGroup {
        let background = &SettingsFile::get(cx).background;
        let (opacity, image_opacity) = (background.opacity, background.image_opacity);
        let opacity = percent_slider(opacity, cx);
        let image_opacity = percent_slider(image_opacity, cx);
        let subscriptions = vec![
            cx.subscribe(&opacity, |_, _, event, cx| {
                slide(event, |b| &mut b.opacity, cx);
            }),
            cx.subscribe(&image_opacity, |_, _, event, cx| {
                slide(event, |b| &mut b.image_opacity, cx);
            }),
        ];
        BackgroundGroup {
            opacity,
            image_opacity,
            _subscriptions: subscriptions,
        }
    }
}

impl Render for BackgroundGroup {
    fn render(&mut self, _: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let background = SettingsFile::get(cx).background.clone();
        let mut rows = vec![style_row(background.style, &theme)];
        match background.style {
            BackgroundStyle::Solid => {}
            BackgroundStyle::Translucent => {
                rows.push(effect_row(background.effect, &theme));
                rows.push(opacity_row(
                    &self.opacity,
                    &t!("settings-appearance-background-opacity-about"),
                    &theme,
                    cx,
                ));
            }
            BackgroundStyle::Image => {
                rows.push(picture_row(&background, &theme, cx));
                if PictureImport::failed(cx) || BackdropImage::failed(cx) {
                    rows.push(form::problem_note(
                        t!("settings-appearance-background-picture-error"),
                        &theme,
                    ));
                }
                rows.push(opacity_row(
                    &self.image_opacity,
                    &t!("settings-appearance-background-image-opacity-about"),
                    &theme,
                    cx,
                ));
            }
        }
        form::group(&t!("settings-appearance-background"), rows, &theme)
    }
}

fn picture_row(background: &Background, theme: &Theme, cx: &App) -> AnyElement {
    let name = background.image.as_ref().map_or_else(
        || t!("settings-appearance-background-picture-none"),
        |image| image.name.clone(),
    );
    let importing = PictureImport::importing(cx);
    let label = if importing {
        t!("settings-appearance-background-importing")
    } else {
        t!("settings-appearance-background-choose")
    };
    let mut choose = button("settings-background-choose", label, Tone::Soft, cx);
    if !importing {
        choose = choose.on_click(|_, _, cx| backdrop::choose_picture(cx));
    }
    let control = div()
        .flex()
        .items_center()
        .gap_3()
        .children(BackdropImage::image(cx).map(|image| {
            img(gpui::ImageSource::Render(image))
                .w(THUMBNAIL_WIDTH)
                .h(THUMBNAIL_HEIGHT)
                .rounded(ROW_RADIUS)
                .object_fit(ObjectFit::Cover)
        }))
        .child(
            div()
                .max_w(px(160.))
                .truncate()
                .text_sm()
                .text_color(theme.muted)
                .child(name),
        )
        .child(choose);
    form::row(
        &t!("settings-appearance-background-picture"),
        Some(&t!("settings-appearance-background-picture-about")),
        control,
        theme,
    )
}

fn style_row(current: BackgroundStyle, theme: &Theme) -> AnyElement {
    let options = vec![
        (
            BackgroundStyle::Solid,
            t!("settings-appearance-background-solid"),
        ),
        (
            BackgroundStyle::Translucent,
            t!("settings-appearance-background-translucent"),
        ),
        (
            BackgroundStyle::Image,
            t!("settings-appearance-background-image"),
        ),
    ];
    let control = choice::segmented(
        "settings-background-style",
        options,
        Some(current),
        false,
        theme,
        choose_style,
    );
    form::row(
        &t!("settings-appearance-background-style"),
        Some(&t!("settings-appearance-background-style-about")),
        control,
        theme,
    )
}

fn effect_row(current: Effect, theme: &Theme) -> AnyElement {
    let options = vec![
        (
            Effect::Frosted,
            t!("settings-appearance-background-frosted"),
        ),
        (Effect::Blur, t!("settings-appearance-background-blur")),
        (Effect::Clear, t!("settings-appearance-background-clear")),
    ];
    let control = choice::segmented(
        "settings-background-effect",
        options,
        Some(current),
        false,
        theme,
        |effect, cx| change(cx, |s| s.background.effect = effect),
    );
    form::row(
        &t!("settings-appearance-background-effect"),
        Some(&t!("settings-appearance-background-effect-about")),
        control,
        theme,
    )
}

fn opacity_row(state: &Entity<SliderState>, about: &str, theme: &Theme, cx: &App) -> AnyElement {
    let percent = state.read(cx).value().end().round();
    let control = div()
        .flex()
        .items_center()
        .gap_3()
        .child(Slider::new(state).w(SLIDER_WIDTH))
        .child(
            div()
                .w(px(40.))
                .text_right()
                .text_sm()
                .text_color(theme.muted)
                .child(t!(
                    "settings-appearance-background-percent",
                    percent = percent
                )),
        );
    form::row(
        &t!("settings-appearance-background-opacity"),
        Some(about),
        control,
        theme,
    )
}

fn percent_slider(value: f32, cx: &mut App) -> Entity<SliderState> {
    cx.new(|_| {
        SliderState::new()
            .min(0.)
            .max(100.)
            .step(1.)
            .default_value(value * 100.)
    })
}

/// A drag shows each step at once but saves only once it is let go.
fn slide(event: &SliderEvent, field: fn(&mut Background) -> &mut f32, cx: &mut App) {
    match *event {
        SliderEvent::Change(value) => {
            let value = value.end() / 100.;
            SettingsFile::update_unsaved(cx, |s| *field(&mut s.background) = value);
        }
        SliderEvent::Release(value) => {
            let value = value.end() / 100.;
            SettingsFile::update(cx, |s| *field(&mut s.background) = value);
        }
    }
    cx.refresh_windows();
}

/// Saves a change to the background and draws every view again, the cached main view
/// included.
fn change(cx: &mut App, edit: impl FnOnce(&mut Settings)) {
    SettingsFile::update(cx, edit);
    cx.refresh_windows();
}

/// Image with no picture yet has nothing to show, so it asks for one right away.
fn choose_style(style: BackgroundStyle, cx: &mut App) {
    change(cx, |s| s.background.style = style);
    if style == BackgroundStyle::Image && SettingsFile::get(cx).background.image.is_none() {
        backdrop::choose_picture(cx);
    }
}
