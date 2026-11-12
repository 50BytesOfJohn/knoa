//! Views. The root shows onboarding until it's done, then the main screen.

pub mod components;
mod main;
pub mod mascot;
mod onboarding;
pub mod theme;
pub mod wordmark;

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _, Window,
    div,
};

pub use main::{OpenSettings, bind_keys, traffic_light_position};

use crate::state::{
    library::Library,
    settings::{SettingsFile, WindowFrame},
};

pub struct RootView {
    library: Entity<Library>,
    onboarding: Option<Entity<onboarding::Onboarding>>,
    main: Option<Entity<main::MainView>>,
}

impl RootView {
    pub fn new(
        library: Entity<Library>,
        window: &mut Window,
        cx: &mut Context<'_, RootView>,
    ) -> RootView {
        cx.observe_window_bounds(window, |_, window, cx| {
            let bounds = window.window_bounds().get_bounds();
            let frame = WindowFrame {
                x: bounds.origin.x.into(),
                y: bounds.origin.y.into(),
                width: bounds.size.width.into(),
                height: bounds.size.height.into(),
            };
            if SettingsFile::get(cx).window != Some(frame) {
                SettingsFile::update(cx, |s| s.window = Some(frame));
            }
        })
        .detach();
        RootView {
            library,
            onboarding: None,
            main: None,
        }
    }
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let screen = if SettingsFile::get(cx).onboarded {
            let wants_account = (self.onboarding.take())
                .is_some_and(|onboarding| onboarding.read(cx).wants_account());
            let library = self.library.clone();
            let main = self
                .main
                .get_or_insert_with(|| cx.new(|cx| main::MainView::new(library, window, cx)))
                .clone();
            if wants_account {
                let main = main.clone();
                cx.defer_in(window, move |_, window, cx| {
                    main.update(cx, |main, cx| main.open_account(window, cx));
                });
            }
            main.into_any_element()
        } else {
            self.main = None;
            let library = self.library.clone();
            self.onboarding
                .get_or_insert_with(|| {
                    cx.new(|cx| onboarding::Onboarding::new(library, window, cx))
                })
                .clone()
                .into_any_element()
        };
        div().size_full().child(screen)
    }
}
