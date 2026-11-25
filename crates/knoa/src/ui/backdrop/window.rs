//! The window's side of a translucent background: GPUI's transparency or blur, our own
//! vibrancy view for Frosted, and gpui-component's root surface, which would cover them
//! all with an opaque fill.

#![expect(
    unsafe_code,
    reason = "GPUI doesn't offer AppKit's vibrancy, only its own neutral blur"
)]

use gpui::{App, Styled as _, Window, WindowBackgroundAppearance};
use objc2::{MainThreadMarker, rc::Retained};
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization as _, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSAutoresizingMaskOptions, NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState, NSVisualEffectView, NSWindowOrderingMode,
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use crate::state::settings::{Background, BackgroundStyle, Effect};

/// What the window itself draws behind Knoa's own canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WindowLook {
    Opaque,
    Clear,
    Blurred,
    /// macOS's vibrancy, in the light or dark appearance to match the theme.
    Frosted {
        dark: bool,
    },
}

impl WindowLook {
    pub(super) fn of(background: &Background, dark: bool) -> WindowLook {
        match (background.style, background.effect) {
            (BackgroundStyle::Solid | BackgroundStyle::Image, _) => WindowLook::Opaque,
            (BackgroundStyle::Translucent, Effect::Frosted) => WindowLook::Frosted { dark },
            (BackgroundStyle::Translucent, Effect::Blur) => WindowLook::Blurred,
            (BackgroundStyle::Translucent, Effect::Clear) => WindowLook::Clear,
        }
    }

    fn appearance(self) -> WindowBackgroundAppearance {
        match self {
            WindowLook::Opaque => WindowBackgroundAppearance::Opaque,
            WindowLook::Blurred => WindowBackgroundAppearance::Blurred,
            WindowLook::Clear | WindowLook::Frosted { .. } => {
                WindowBackgroundAppearance::Transparent
            }
        }
    }

    fn is_opaque(self) -> bool {
        self == WindowLook::Opaque
    }
}

/// The look last applied, so a setting written again changes nothing.
pub(super) struct WindowBackdrop {
    applied: WindowLook,
    frosted: Option<Retained<NSVisualEffectView>>,
}

impl WindowBackdrop {
    /// GPUI opens windows opaque, and the root surface starts with its fill.
    pub(super) fn new() -> WindowBackdrop {
        WindowBackdrop {
            applied: WindowLook::Opaque,
            frosted: None,
        }
    }

    pub(super) fn apply(&mut self, look: WindowLook, window: &mut Window, cx: &mut App) {
        if look == self.applied {
            return;
        }
        let before = std::mem::replace(&mut self.applied, look);
        if before.appearance() != look.appearance() {
            window.set_background_appearance(look.appearance());
        }
        match look {
            WindowLook::Frosted { dark } => self.frost(dark, window),
            WindowLook::Opaque | WindowLook::Clear | WindowLook::Blurred => self.unfrost(),
        }
        if before.is_opaque() != look.is_opaque() {
            fill_root(look.is_opaque(), window, cx);
        }
    }

    fn frost(&mut self, dark: bool, window: &Window) {
        if self.frosted.is_none() {
            self.frosted = insert_effect_view(window);
        }
        let Some(view) = &self.frosted else {
            return;
        };
        // SAFETY: AppKit's appearance names are constants that live for the whole process.
        let name = unsafe {
            if dark {
                NSAppearanceNameDarkAqua
            } else {
                NSAppearanceNameAqua
            }
        };
        view.setAppearance(NSAppearance::appearanceNamed(name).as_deref());
    }

    fn unfrost(&mut self) {
        if let Some(view) = self.frosted.take() {
            view.removeFromSuperview();
        }
    }
}

/// An `NSVisualEffectView` under everything in the window, the way GPUI adds its own
/// blur, but left as `AppKit` draws it, tinted by the desktop.
fn insert_effect_view(window: &Window) -> Option<Retained<NSVisualEffectView>> {
    let mtm = MainThreadMarker::new()?;
    // GPUI's own `window_handle` names the window, not its NSView.
    let handle = HasWindowHandle::window_handle(window).ok()?;
    let RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
        return None;
    };
    // SAFETY: the handle points at GPUI's NSView, which lives as long as the window does,
    // and the window outlives this borrow.
    let gpui_view: &NSView = unsafe { appkit.ns_view.cast().as_ref() };
    let content = gpui_view.window()?.contentView()?;
    let view = NSVisualEffectView::initWithFrame(mtm.alloc(), content.bounds());
    view.setMaterial(NSVisualEffectMaterial::UnderWindowBackground);
    view.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
    view.setState(NSVisualEffectState::Active);
    view.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    content.addSubview_positioned_relativeTo(&view, NSWindowOrderingMode::Below, None);
    Some(view)
}

/// gpui-component paints the root surface with its theme's background. Clearing it on
/// the root itself, rather than in the theme, leaves every component's colours alone.
fn fill_root(opaque: bool, window: &mut Window, cx: &mut App) {
    let Some(Some(root)) = window.root::<gpui_component::Root>() else {
        return;
    };
    root.update(cx, |root, cx| {
        root.style().background = (!opaque).then(|| gpui::transparent_black().into());
        cx.notify();
    });
}
