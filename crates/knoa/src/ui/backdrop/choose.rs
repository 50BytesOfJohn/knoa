//! Choosing a picture for the background: the open panel, the copy, and the setting that
//! shows it. The app owns all of it, so closing Settings meanwhile loses nothing.

use std::path::PathBuf;

use gpui::{App, AppContext as _, Global, PathPromptOptions};

use super::import;
use crate::{
    paths::AppPaths,
    state::settings::{BackgroundImage, SettingsFile},
};

#[derive(Default)]
pub struct PictureImport {
    importing: bool,
    failed: bool,
}

impl Global for PictureImport {}

impl PictureImport {
    /// A chosen picture is being copied.
    pub fn importing(cx: &App) -> bool {
        cx.try_global::<PictureImport>()
            .is_some_and(|state| state.importing)
    }

    /// The last picture chosen couldn't be copied.
    pub fn failed(cx: &App) -> bool {
        cx.try_global::<PictureImport>()
            .is_some_and(|state| state.failed)
    }
}

/// Asks for a picture, copies it into Knoa's folder off the UI thread, and shows it.
pub fn choose_picture(cx: &mut App) {
    if PictureImport::importing(cx) {
        return;
    }
    let picked = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: false,
        multiple: false,
        prompt: None,
    });
    let folder = cx.global::<AppPaths>().0.support.clone();
    cx.spawn(async move |cx| {
        let source = match picked.await {
            Ok(Ok(Some(paths))) => paths.into_iter().next(),
            Ok(Ok(None)) | Err(_) => None,
            Ok(Err(err)) => {
                tracing::warn!("the open panel failed: {err}");
                None
            }
        };
        let Some(source) = source else {
            return;
        };
        cx.update(|cx| set_state(true, false, cx));
        let name = source
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let copied = cx
            .background_spawn(async move { import::import(&source, &folder) })
            .await;
        cx.update(|cx| finish(copied, name, cx));
    })
    .detach();
}

fn finish(copied: anyhow::Result<PathBuf>, name: String, cx: &mut App) {
    match copied {
        Ok(path) => {
            let image = BackgroundImage { path, name };
            SettingsFile::update(cx, |s| s.background.image = Some(image));
            set_state(false, false, cx);
        }
        Err(err) => {
            tracing::warn!("can't use the picture as a background: {err:#}");
            set_state(false, true, cx);
        }
    }
}

/// Draws every view again, so Settings shows the change if it is open.
fn set_state(importing: bool, failed: bool, cx: &mut App) {
    *cx.default_global::<PictureImport>() = PictureImport { importing, failed };
    cx.refresh_windows();
}
