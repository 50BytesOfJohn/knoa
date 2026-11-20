//! The menu bar, and the app-wide actions only it triggers.

use gpui::{App, KeyBinding, Menu, MenuItem, SystemMenuType, actions};
use objc2_foundation::NSProcessInfo;

use crate::{app::app_name, i18n::t, services::updates::Updates, ui::OpenSettings};

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const VERSION: &str = env!("CARGO_PKG_VERSION");

actions!(
    knoa,
    [
        About,
        CheckForUpdates,
        Hide,
        HideOthers,
        ShowAll,
        Quit,
        ReleaseNotes,
        ReportIssue
    ]
);

pub fn init(cx: &mut App) {
    cx.on_action(|_: &About, _| about::show());
    cx.on_action(|_: &Hide, cx| cx.hide());
    cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
    cx.on_action(|_: &ShowAll, cx| cx.unhide_other_apps());
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &ReleaseNotes, cx| {
        cx.open_url(&format!("{REPOSITORY}/releases/tag/v{VERSION}"));
    });
    cx.on_action(|_: &ReportIssue, cx| cx.open_url(&issue_url()));
    // Without Sparkle there's no handler, so GPUI greys the menu item out.
    if Updates::get(cx).is_some() {
        cx.on_action(|_: &CheckForUpdates, cx| {
            if let Some(updates) = Updates::get(cx) {
                updates.check_now();
            }
        });
    }

    cx.bind_keys([
        KeyBinding::new("cmd-h", Hide, None),
        KeyBinding::new("alt-cmd-h", HideOthers, None),
        KeyBinding::new("cmd-q", Quit, None),
    ]);
    set(cx);
}

/// Sets the menu bar in the current language.
pub fn set(cx: &mut App) {
    cx.set_menus(vec![app_menu(), help_menu()]);
}

fn app_menu() -> Menu {
    let name = app_name();
    Menu::new(name).items([
        MenuItem::action(t!("menu-about", name = name), About),
        MenuItem::action(t!("menu-check-for-updates"), CheckForUpdates),
        MenuItem::separator(),
        MenuItem::action(t!("menu-settings"), OpenSettings),
        MenuItem::separator(),
        MenuItem::os_submenu(t!("menu-services"), SystemMenuType::Services),
        MenuItem::separator(),
        MenuItem::action(t!("menu-hide", name = name), Hide),
        MenuItem::action(t!("menu-hide-others"), HideOthers),
        MenuItem::action(t!("menu-show-all"), ShowAll),
        MenuItem::separator(),
        MenuItem::action(t!("menu-quit", name = name), Quit),
    ])
}

fn help_menu() -> Menu {
    let name = app_name();
    Menu::new(t!("menu-help")).items([
        MenuItem::action(
            t!("menu-whats-new", name = name, version = VERSION),
            ReleaseNotes,
        ),
        MenuItem::action(t!("menu-report-issue"), ReportIssue),
    ])
}

/// A new bug report with the version fields of the issue form filled in.
fn issue_url() -> String {
    let macos = NSProcessInfo::processInfo()
        .operatingSystemVersionString()
        .to_string();
    let base = format!("{REPOSITORY}/issues/new");
    let params = [
        ("template", "bug_report.yml"),
        ("version", VERSION),
        ("macos", macos.as_str()),
    ];
    match url::Url::parse_with_params(&base, params) {
        Ok(url) => url.into(),
        Err(_) => base,
    }
}

mod about {
    #![expect(
        unsafe_code,
        reason = "GPUI doesn't wrap AppKit's standard About panel"
    )]

    use objc2::{
        msg_send,
        rc::Retained,
        runtime::{AnyClass, AnyObject},
    };

    /// `AppKit`'s About panel: icon, name, version and copyright from the bundle's `Info.plist`.
    pub fn show() {
        let Some(class) = AnyClass::get(c"NSApplication") else {
            return;
        };
        let none: Option<&AnyObject> = None;
        // SAFETY: +[NSApplication sharedApplication] returns the running app, and
        // -orderFrontStandardAboutPanel: takes a nullable sender. GPUI calls actions on
        // the main thread.
        unsafe {
            let app: Retained<AnyObject> = msg_send![class, sharedApplication];
            let () = msg_send![&*app, orderFrontStandardAboutPanel: none];
        }
    }
}
