//! Application bootstrap: state, services, actions, menus and the main window.

use std::sync::Arc;

use anyhow::Context as _;
use gpui::{
    App, AppContext as _, Bounds, TitlebarOptions, WindowBounds, WindowOptions, point, px, size,
};

use crate::{
    paths::Paths,
    services::{
        ai, cloud::Cloud, credentials::Credentials, housekeeping, network::Network,
        organizer::Organizer, sync, thumbnails::Thumbnails, updates::Updates,
    },
    state::{
        database::Database,
        library::Library,
        settings::{self, SettingsFile},
    },
    ui::{RootView, theme::Theme},
};

pub const APP_NAME: &str = "Knoa";
const APP_ID: &str = "dev.knoa.Knoa";

pub fn init(cx: &mut App) -> anyhow::Result<()> {
    gpui_component::init(cx);
    let paths = Paths::new()?;
    cx.set_global(SettingsFile::load(paths.settings()));
    Theme::apply(&SettingsFile::get(cx).theme.clone(), cx);
    Credentials::init(paths.dev_credentials(), cx);
    Network::init(cx)?;

    let database = Database::open(&paths.database())
        .with_context(|| format!("opening {}", paths.database().display()))?;
    settings::adopt_legacy(&mut database.write(), cx)
        .context("moving settings into the workspace")?;
    let library = cx.new(|cx| Library::new(Arc::new(database), cx));
    housekeeping::run(&library, paths.thumbnails(), cx);
    sync::init(&library, cx);
    let fetcher = Arc::new(knoa_fetch::Fetcher::new(paths.thumbnails())?);
    Organizer::init(library.clone(), fetcher.clone(), cx);
    // Before the organizer's first round, so it knows whether the cloud has the queue.
    Cloud::init(library.clone(), fetcher.clone(), cx)?;
    Thumbnails::init(library.clone(), fetcher, cx);
    ai::connect(cx);
    cx.set_global(paths);

    Updates::init(cx);
    crate::menus::init(cx);
    crate::ui::bind_keys(cx);

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
            traffic_light_position: Some(crate::ui::traffic_light_position()),
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
