//! The app's assets: Knoa's own icons, compiled in, over the Lucide set.

use std::borrow::Cow;

use gpui::{AssetSource, SharedString};

/// The asset path of one of Knoa's own icons. They sit apart from Lucide's, which use
/// some of the same names.
pub fn knoa_icon(name: &str) -> SharedString {
    format!("icons/knoa/{name}.svg").into()
}

macro_rules! knoa_icons {
    ($($name:literal),+ $(,)?) => {
        &[$((
            concat!("icons/knoa/", $name, ".svg"),
            include_bytes!(concat!("../assets/icons/knoa/", $name, ".svg")),
        )),+]
    };
}

/// Every Knoa icon by asset path. The folder glyphs' names come from `FolderIcon`.
const ICONS: &[(&str, &[u8])] = knoa_icons![
    "folder",
    "folder-filled",
    "read",
    "article",
    "code",
    "write",
    "work",
    "tag",
    "bag",
    "plane",
    "cup",
    "music",
    "play",
    "photo",
    "bulb",
    "heart",
    "home",
    "star",
    "clock",
    "flask",
    "globe",
    "person",
    "box",
    "inbox",
    "leaf",
    "bolt",
    "lock",
    "pin",
    "chat",
    "game",
];

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        match ICONS.iter().find(|(icon, _)| *icon == path) {
            Some((_, data)) => Ok(Some(Cow::Borrowed(data))),
            None => gpui_kit_assets::AllAssets.load(path),
        }
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let mut paths = gpui_kit_assets::AllAssets.list(path)?;
        paths.extend(
            ICONS
                .iter()
                .filter(|(icon, _)| icon.starts_with(path))
                .map(|(icon, _)| SharedString::from(*icon)),
        );
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use knoa_core::FolderIcon;

    use super::*;

    // The glyph names live in knoa-core and the files here; a glyph without its file
    // would draw as nothing, with only a log line to say so.
    #[test]
    fn every_folder_glyph_has_its_icon() {
        for glyph in FolderIcon::ALL {
            let path = knoa_icon(glyph.name());
            assert!(matches!(Assets.load(&path), Ok(Some(_))), "{path}");
        }
    }
}
