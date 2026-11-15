//! A bookmark's stored picture as a file: copied to the clipboard as an image, or saved
//! where the user picks. Both use the display copy Knoa keeps; the original stays at the
//! bookmark's link.

use std::path::{Path, PathBuf};

use gpui::{AppContext as _, ClipboardItem, Context, Image, ImageFormat};
use knoa_core::{Bookmark, BookmarkId, Kind};

use super::MainView;

/// Long titles make unwieldy file names; the save panel lets the user change it anyway.
const NAME_CHARS: usize = 80;

impl MainView {
    /// The stored picture of a bookmark whose kind is a picture, if Knoa kept one.
    pub(super) fn picture_of(&self, id: BookmarkId, cx: &gpui::App) -> Option<PathBuf> {
        let bookmark = self.library.read(cx).bookmark(id)?;
        let image = bookmark.image.as_ref().filter(|_| has_picture(bookmark))?;
        Some(image.path.clone())
    }

    pub(super) fn copy_image_of(&mut self, id: BookmarkId, cx: &mut Context<'_, MainView>) {
        let Some(path) = self.picture_of(id, cx) else {
            return;
        };
        let read = cx.background_spawn(async move { read_image(&path) });
        cx.spawn(async move |this, cx| {
            let image = read.await;
            let _ = this.update(cx, |this, cx| match image {
                Ok(image) => {
                    cx.write_to_clipboard(ClipboardItem::new_image(&image));
                    this.tell_viewer("Image copied", cx);
                }
                Err(err) => {
                    tracing::warn!(?id, "can't copy the image: {err}");
                    this.tell_viewer("Couldn't copy the image", cx);
                }
            });
        })
        .detach();
    }

    /// Asks where to save the picture, in Downloads under the bookmark's name by default.
    pub(super) fn save_image_of(&mut self, id: BookmarkId, cx: &mut Context<'_, MainView>) {
        let (Some(path), Some(title)) = (
            self.picture_of(id, cx),
            self.library.read(cx).bookmark(id).map(name_of),
        ) else {
            return;
        };
        let folder = dirs::download_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_default();
        let name = file_name(&title, &path);
        let chosen = cx.prompt_for_new_path(&folder, Some(&name));
        cx.spawn(async move |this, cx| {
            let target = match chosen.await {
                Ok(Ok(Some(target))) => target,
                Ok(Ok(None)) | Err(_) => return,
                Ok(Err(err)) => {
                    tracing::warn!("the save panel failed: {err}");
                    return;
                }
            };
            let copy = cx.background_spawn(async move { std::fs::copy(&path, &target) });
            let outcome = copy.await;
            let _ = this.update(cx, |this, cx| match outcome {
                Ok(_) => this.tell_viewer("Image saved", cx),
                Err(err) => {
                    tracing::warn!(?id, "can't save the image: {err}");
                    this.tell_viewer("Couldn't save the image", cx);
                }
            });
        })
        .detach();
    }
}

/// Whether the bookmark is a picture Knoa kept a copy of, which the viewer can show.
pub(super) fn has_picture(bookmark: &Bookmark) -> bool {
    let media = bookmark
        .kind
        .is_some_and(|kind| Kind::MEDIA.contains(&kind));
    media && bookmark.image.is_some()
}

fn read_image(path: &Path) -> anyhow::Result<Image> {
    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(format_of)
        .ok_or_else(|| anyhow::anyhow!("unknown image format"))?;
    Ok(Image::from_bytes(format, std::fs::read(path)?))
}

/// Stored copies are named by their format, which Knoa chose when it made them.
fn format_of(extension: &str) -> Option<ImageFormat> {
    match extension.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "png" => Some(ImageFormat::Png),
        "gif" => Some(ImageFormat::Gif),
        "webp" => Some(ImageFormat::Webp),
        _ => None,
    }
}

/// The title, or while the title is still the link, the link's file name.
fn name_of(bookmark: &Bookmark) -> String {
    if bookmark.title != bookmark.url {
        return bookmark.title.clone();
    }
    url::Url::parse(&bookmark.url)
        .ok()
        .and_then(|url| Some(url.path_segments()?.next_back()?.to_owned()))
        .filter(|segment| !segment.is_empty())
        .unwrap_or_default()
}

/// The title as a file name, with the stored copy's extension. A title that is itself a
/// file name ("wallpaper.png") loses its own extension, which may no longer be right.
fn file_name(title: &str, stored: &Path) -> String {
    let extension = stored.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
    let stem = match title.rsplit_once('.') {
        Some((stem, own)) if format_of(own).is_some() => stem,
        _ => title,
    };
    let stem: String = stem
        .chars()
        .map(|c| if matches!(c, '/' | ':') { '-' } else { c })
        .take(NAME_CHARS)
        .collect();
    let stem = stem.trim();
    let stem = if stem.is_empty() { "Image" } else { stem };
    format!("{stem}.{extension}")
}
