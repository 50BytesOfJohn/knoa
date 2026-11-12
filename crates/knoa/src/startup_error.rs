//! Telling the user why Knoa couldn't start, in a native alert, then quitting.

use gpui::App;
use objc2::MainThreadMarker;
use objc2_app_kit::NSAlert;
use objc2_foundation::NSString;

use crate::app::app_name;

/// Shows `err` in an app-modal alert and quits once the user dismisses it.
///
/// Uses a bare `NSAlert`: `init` may have failed before the theme, settings or any window
/// existed, and a launch from Finder has no terminal to read a log in.
pub fn report(err: &anyhow::Error, cx: &mut App) {
    tracing::error!("failed to start: {err:#}");
    let message = format!("{err:#}");
    cx.activate(true);
    // The alert's modal loop keeps running GPUI's main-thread tasks, and a task from a
    // half-done `init` would panic borrowing the app if `cx` were still held here. So
    // show it from a task of its own, as GPUI does for its own alerts.
    cx.spawn(async move |cx| {
        run_alert(&message);
        cx.update(|cx| cx.quit());
    })
    .detach();
}

fn run_alert(message: &str) {
    let mtm = MainThreadMarker::new().expect("GPUI runs foreground tasks on the main thread");
    let alert = NSAlert::new(mtm);
    alert.setMessageText(&NSString::from_str(&format!("{} can't start", app_name())));
    alert.setInformativeText(&NSString::from_str(message));
    alert.addButtonWithTitle(&NSString::from_str("Quit"));
    alert.runModal();
}
