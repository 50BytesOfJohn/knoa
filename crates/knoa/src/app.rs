//! Application bootstrap: state, services, actions, menus and the main window.

use std::sync::{Arc, Mutex};

use anyhow::Context as _;
use gpui::{
    App, AppContext as _, Bounds, KeyBinding, Menu, MenuItem, TitlebarOptions, WindowBounds,
    WindowOptions, actions, point, px, size,
};

use crate::{
    paths::Paths,
    services::{ai, credentials::Credentials, network::Network, organizer::Organizer, sync},
    state::{library::Library, settings::SettingsFile},
    ui::{RootView, theme::Theme},
};

pub const APP_NAME: &str = "Knoa";
const APP_ID: &str = "dev.knoa.Knoa";

actions!(knoa, [Quit]);

pub fn init(cx: &mut App) -> anyhow::Result<()> {
    gpui_component::init(cx);
    let paths = Paths::new()?;
    cx.set_global(SettingsFile::load(paths.settings()));
    Theme::apply(&SettingsFile::get(cx).theme.clone(), cx);
    Credentials::init(paths.dev_credentials(), cx);
    Network::init(cx)?;

    let store = knoa_store::Store::open(&paths.database())
        .with_context(|| format!("opening {}", paths.database().display()))?;
    let library = cx.new(|cx| Library::new(Arc::new(Mutex::new(store)), cx));
    Organizer::init(
        library.clone(),
        knoa_fetch::Fetcher::new(paths.thumbnails())?,
        cx,
    );
    ai::connect(cx);
    if SettingsFile::get(cx).onboarded {
        sync::restart(library.clone(), cx);
    }

    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
    crate::ui::bind_keys(cx);
    cx.set_menus(vec![
        Menu::new(APP_NAME).items([MenuItem::action(format!("Quit {APP_NAME}"), Quit)]),
    ]);

    // A single-window app: closing it should end the process.
    cx.on_window_closed(|cx, _window_id| {
        if cx.windows().is_empty() {
            cx.quit();
        }
    })
    .detach();

    open_main_window(library, cx)?;
    cx.activate(true);
    Ok(())
}

fn open_main_window(library: gpui::Entity<Library>, cx: &mut App) -> anyhow::Result<()> {
    let bounds = match SettingsFile::get(cx).window {
        Some(frame) => Bounds::new(
            point(px(frame.x), px(frame.y)),
            size(px(frame.width), px(frame.height)),
        ),
        None => Bounds::centered(None, size(px(1280.), px(820.)), cx),
    };
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(900.), px(560.))),
        titlebar: Some(TitlebarOptions {
            title: Some(APP_NAME.into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(26.), px(26.))),
        }),
        app_id: Some(APP_ID.into()),
        ..Default::default()
    };

    cx.open_window(options, |window, cx| {
        let root = cx.new(|cx| RootView::new(library, window, cx));
        cx.new(|cx| gpui_component::Root::new(root, window, cx))
    })
    .context("opening the main window")?;
    Ok(())
}
