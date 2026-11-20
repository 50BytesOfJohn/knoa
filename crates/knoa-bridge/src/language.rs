//! The Mac app's language, so the extension can speak it while signed out.

use std::path::Path;

use knoa_core::Language;
use serde::Deserialize;

/// The one field of the app's settings file the bridge needs. Everything else in the
/// file is ignored, so the app can add settings without breaking the bridge.
#[derive(Deserialize)]
struct Settings {
    #[serde(default)]
    language: Language,
}

/// The language in the settings file at `path`. English when the file is missing,
/// unreadable, or holds no language Knoa knows, like the app itself.
pub fn read(path: &Path) -> Language {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Settings>(&bytes).ok())
        .map(|settings| settings.language)
        .unwrap_or_default()
}
