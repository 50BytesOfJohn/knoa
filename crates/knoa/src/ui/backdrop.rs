//! The main window's background style: what shows behind the floating panels. The
//! window's transparency and material change only when the setting or the theme does;
//! the main view paints the canvas over them each frame with [`canvas_color`] and
//! [`picture`].

mod choose;
mod image;
mod import;
mod window;

use std::path::{Path, PathBuf};

use gpui::{
    App, AppContext as _, ImageSource, IntoElement, ObjectFit, Styled as _, StyledImage as _,
    Window, img,
};

pub use choose::{PictureImport, choose_picture};
pub use image::BackdropImage;
use window::{WindowBackdrop, WindowLook};

use crate::{
    state::settings::{BackgroundStyle, SettingsFile},
    ui::theme::Theme,
};

/// The least canvas a translucent window is painted with: macOS sends clicks on fully
/// transparent pixels of a non-opaque window to whatever is behind it.
const MIN_ALPHA: f32 = 0.01;

/// Keeps the window in step with the background setting and the theme. Owned by the view
/// that owns the window.
pub struct Backdrop {
    window: WindowBackdrop,
}

impl Backdrop {
    pub fn new() -> Backdrop {
        Backdrop {
            window: WindowBackdrop::new(),
        }
    }

    /// Applies what changed since the last call. Most settings writes change nothing
    /// here, so this returns after a few comparisons.
    pub fn sync(&mut self, window: &mut Window, cx: &mut App) {
        let background = &SettingsFile::get(cx).background;
        let look = WindowLook::of(background, Theme::get(cx).dark);
        let shown = background.shown_image();
        let new_image = (!BackdropImage::shows(shown, cx)).then(|| shown.map(Path::to_path_buf));
        self.window.apply(look, window, cx);
        if let Some(path) = new_image {
            image::show(path, window, cx);
        }
    }
}

/// Removes the copies of pictures that the settings no longer show: replaced ones, and
/// any left half-made. Done once at launch, rather than on replacing, so a settings file
/// that failed to save still finds the picture it names.
pub fn remove_unused_copies(dir: PathBuf, cx: &App) {
    let in_use = SettingsFile::get(cx)
        .background
        .image
        .as_ref()
        .map(|image| image.path.clone());
    cx.background_spawn(async move {
        match import::remove_unused(&dir, in_use.as_deref()) {
            Ok(removed) => tracing::info!(removed, "cleaned up background pictures"),
            Err(err) => tracing::warn!("can't clean up background pictures: {err}"),
        }
    })
    .detach();
}

/// The colour behind the panels: the theme's canvas, see-through while the window is.
pub fn canvas_color(theme: &Theme, cx: &App) -> gpui::Hsla {
    let background = &SettingsFile::get(cx).background;
    match background.style {
        BackgroundStyle::Solid | BackgroundStyle::Image => theme.canvas,
        BackgroundStyle::Translucent => theme.canvas.alpha(background.opacity.max(MIN_ALPHA)),
    }
}

/// The picture filling the window behind the panels, while there is one to show.
pub fn picture(cx: &App) -> Option<impl IntoElement> {
    let background = &SettingsFile::get(cx).background;
    if background.style != BackgroundStyle::Image {
        return None;
    }
    let image = BackdropImage::image(cx)?;
    Some(
        img(ImageSource::Render(image))
            .absolute()
            .inset_0()
            .size_full()
            .object_fit(ObjectFit::Cover)
            .opacity(background.image_opacity),
    )
}
