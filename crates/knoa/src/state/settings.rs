//! User settings, kept as a small JSON file next to the database.

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use gpui::{App, Global};
use knoa_ai::ProviderConfig;
use knoa_core::{Ink, Language, protocol::Account, unknown_language_as_default};
use knoa_store::Store;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Onboarding finished or skipped.
    pub onboarded: bool,
    /// The workspace shown, by id.
    pub current_workspace: Option<i64>,
    /// `None` when the user runs Knoa without AI.
    pub ai: Option<ProviderConfig>,
    pub theme: String,
    /// The UI's language and the AI's. Signed in, the account's language wins.
    #[serde(deserialize_with = "unknown_language_as_default")]
    pub language: Language,
    /// The user picked `language` here while signed in, and the account doesn't have it
    /// yet. Until it does, the account's language doesn't replace it.
    pub language_unsent: bool,
    pub show_up_next: bool,
    pub show_summaries: bool,
    /// How the Events place shows events on this Mac.
    pub events_view: EventsView,
    /// Sidebar folders shown collapsed, by local id. Kept on this Mac only.
    pub collapsed_folders: BTreeSet<i64>,
    pub window: Option<WindowFrame>,
    /// The signed-in cloud account as last seen, so the app knows who organizes before
    /// the server answers. The token itself is with the credentials.
    pub account: Option<Account>,
    #[serde(flatten, skip_serializing)]
    pub legacy: Legacy,
}

/// Settings from before workspaces, read once and moved into the first workspace.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Legacy {
    pub workspace: Option<LegacyWorkspace>,
    pub browsers: Vec<String>,
    pub instructions: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LegacyWorkspace {
    pub name: String,
    pub color: u8,
}

impl Legacy {
    fn is_empty(&self) -> bool {
        *self == Legacy::default()
    }
}

/// How the Events place lays out its events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventsView {
    /// A month grid, with the upcoming events beside it.
    #[default]
    Calendar,
    /// One list: happening now, upcoming by day, undated, then past.
    List,
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
            current_workspace: None,
            ai: None,
            theme: crate::ui::theme::DEFAULT_THEME.into(),
            language: Language::default(),
            language_unsent: false,
            show_up_next: true,
            show_summaries: true,
            events_view: EventsView::default(),
            collapsed_folders: BTreeSet::new(),
            window: None,
            account: None,
            legacy: Legacy::default(),
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

/// Moves pre-workspace settings into the first workspace, once.
pub fn adopt_legacy(store: &mut Store, cx: &mut App) -> knoa_store::Result<()> {
    let legacy = SettingsFile::get(cx).legacy.clone();
    if legacy.is_empty() {
        return Ok(());
    }
    if let Some(mut workspace) = store.workspaces()?.into_iter().next() {
        if let Some(old) = legacy.workspace {
            workspace.name = old.name;
            workspace.color = old_palette_ink(old.color);
        }
        workspace.browsers = legacy.browsers;
        workspace.instructions = legacy.instructions;
        store.update_workspace(&workspace)?;
        tracing::info!(
            "moved browsers and instructions into workspace {}",
            workspace.name
        );
    }
    SettingsFile::update(cx, |s| s.legacy = Legacy::default());
    Ok(())
}

/// The ink nearest a color from the palette of eight these settings were saved with, the
/// same mapping the database's migration to twelve inks applies.
fn old_palette_ink(slot: u8) -> Ink {
    let slot = usize::from(slot % 8);
    Ink::nth(if slot < 3 { slot } else { slot + 1 })
}

/// Writes to a temporary file and renames it, so a crash never leaves half a file.
fn save(path: &Path, settings: &Settings) -> anyhow::Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(settings)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
