//! The picture behind the panels, decoded once off the UI thread and held until another
//! replaces it or the style stops showing one, when GPUI is told to let it go.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use gpui::{App, AppContext as _, Global, RenderImage, Window};
use image::{DynamicImage, Frame, ImageDecoder as _, ImageReader};

#[derive(Default)]
pub struct BackdropImage {
    /// The copy shown, or being decoded to be.
    path: Option<PathBuf>,
    image: Option<Arc<RenderImage>>,
    failed: bool,
}

impl Global for BackdropImage {}

impl BackdropImage {
    pub fn image(cx: &App) -> Option<Arc<RenderImage>> {
        cx.try_global::<BackdropImage>()?.image.clone()
    }

    /// The picture asked for couldn't be decoded, so the canvas shows instead.
    pub fn failed(cx: &App) -> bool {
        cx.try_global::<BackdropImage>()
            .is_some_and(|state| state.failed)
    }

    pub(super) fn shows(path: Option<&Path>, cx: &App) -> bool {
        cx.try_global::<BackdropImage>()
            .and_then(|state| state.path.as_deref())
            == path
    }
}

/// Shows the picture at `path`, or none. The picture shown until now stays until the new
/// one is decoded, so replacing it never flashes the canvas.
pub(super) fn show(path: Option<PathBuf>, window: &mut Window, cx: &mut App) {
    let state = cx.default_global::<BackdropImage>();
    state.path.clone_from(&path);
    state.failed = false;
    let Some(path) = path else {
        release(window, cx);
        return;
    };
    let decode = cx.background_spawn({
        let path = path.clone();
        async move { decode(&path) }
    });
    cx.spawn(async move |cx| {
        let decoded = decode.await;
        cx.update(|cx| finish(&path, decoded, cx));
    })
    .detach();
}

fn finish(path: &Path, decoded: anyhow::Result<RenderImage>, cx: &mut App) {
    let state = cx.default_global::<BackdropImage>();
    // Another picture was asked for meanwhile. This one never reached GPUI, so dropping
    // it frees it.
    if state.path.as_deref() != Some(path) {
        return;
    }
    let old = match decoded {
        Ok(image) => state.image.replace(Arc::new(image)),
        Err(err) => {
            tracing::warn!(
                "can't show the background picture {}: {err:#}",
                path.display()
            );
            state.failed = true;
            state.image.take()
        }
    };
    if let Some(old) = old {
        cx.drop_image(old, None);
    }
    cx.refresh_windows();
}

/// Lets go of the picture shown, in memory and in the window's atlas.
fn release(window: &mut Window, cx: &mut App) {
    if let Some(old) = cx.default_global::<BackdropImage>().image.take() {
        cx.drop_image(old, Some(window));
        cx.refresh_windows();
    }
}

fn decode(path: &Path) -> anyhow::Result<RenderImage> {
    let mut decoder = ImageReader::open(path)?
        .with_guessed_format()?
        .into_decoder()?;
    let orientation = decoder.orientation()?;
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);
    let mut pixels = image.into_rgba8();
    // GPUI's atlas takes BGRA.
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
    }
    Ok(RenderImage::new([Frame::new(pixels)]))
}
