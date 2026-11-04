//! Application bootstrap: global actions, key bindings, menus and the main window.

use anyhow::Context as _;
use gpui::{
    App, AppContext as _, Bounds, KeyBinding, Menu, MenuItem, TitlebarOptions, WindowBounds,
    WindowOptions, actions, point, px, size,
};

use crate::ui::RootView;

pub const APP_NAME: &str = "Knoa";
const APP_ID: &str = "dev.knoa.Knoa";

actions!(knoa, [Quit]);

pub fn init(cx: &mut App) -> anyhow::Result<()> {
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
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

    open_main_window(cx)?;
    cx.activate(true);
    Ok(())
}

fn open_main_window(cx: &mut App) -> anyhow::Result<()> {
    let bounds = Bounds::centered(None, size(px(1100.), px(720.)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(640.), px(420.))),
        titlebar: Some(TitlebarOptions {
            title: Some(APP_NAME.into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(12.), px(12.))),
        }),
        app_id: Some(APP_ID.into()),
        ..Default::default()
    };

    cx.open_window(options, |_window, cx| cx.new(|_cx| RootView::new()))
        .context("opening the main window")?;
    Ok(())
}
