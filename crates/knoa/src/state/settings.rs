//! User settings, kept as a small JSON file next to the database.

use std::path::{Path, PathBuf};

use gpui::{App, Global};
use knoa_ai::ProviderConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Onboarding finished or skipped.
    pub onboarded: bool,
    pub workspace: Workspace,
    /// Browser profiles to sync, as `helium:Default`.
    pub browsers: Vec<String>,
    /// `None` when the user runs Knoa without AI.
    pub ai: Option<ProviderConfig>,
    /// How the user wants things organized, in their own words. Sent with every request.
    pub instructions: String,
    pub theme: String,
    pub show_up_next: bool,
    pub show_summaries: bool,
    pub window: Option<WindowFrame>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub color: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            onboarded: false,
            workspace: Workspace {
                name: "Personal".into(),
                color: 0,
            },
            browsers: Vec::new(),
            ai: None,
            instructions: String::new(),
            theme: crate::ui::theme::DEFAULT_THEME.into(),
            show_up_next: true,
            show_summaries: true,
            window: None,
        }
    }
}

/// The settings and the file they live in.
pub struct SettingsFile {
    pub settings: Settings,
    path: PathBuf,
}

impl Global for SettingsFile {}

impl SettingsFile {
    /// Loads settings, falling back to defaults when the file is missing or unreadable.
    pub fn load(path: PathBuf) -> SettingsFile {
        let settings = match std::fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|err| {
                tracing::warn!("settings file is invalid, using defaults: {err}");
                Settings::default()
            }),
            Err(_) => Settings::default(),
        };
        SettingsFile { settings, path }
    }

    pub fn get(cx: &App) -> &Settings {
        &cx.global::<SettingsFile>().settings
    }

    /// Changes settings and saves them. The write is small, so it happens right away.
    pub fn update(cx: &mut App, change: impl FnOnce(&mut Settings)) {
        let file = cx.global_mut::<SettingsFile>();
        change(&mut file.settings);
        if let Err(err) = save(&file.path, &file.settings) {
            tracing::error!("can't save settings: {err:#}");
        }
    }
}

/// Writes to a temporary file and renames it, so a crash never leaves half a file.
fn save(path: &Path, settings: &Settings) -> anyhow::Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(settings)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
