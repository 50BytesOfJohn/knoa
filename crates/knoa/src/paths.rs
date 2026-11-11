//! Where Knoa keeps its files.

use std::path::PathBuf;

use anyhow::Context as _;
use gpui::Global;

#[derive(Debug, Clone)]
pub struct Paths {
    /// `~/Library/Application Support/Knoa`, or `Knoa Dev` for debug builds: the database,
    /// settings and dev credentials.
    pub support: PathBuf,
    /// `~/Library/Caches/Knoa` (or `Knoa Dev`): thumbnails, safe to delete.
    pub cache: PathBuf,
}

impl Global for Paths {}

/// Debug builds keep their own library. A branch's newer schema would otherwise migrate
/// the installed release's database, which then refuses to open it.
const FOLDER: &str = if cfg!(debug_assertions) {
    "Knoa Dev"
} else {
    "Knoa"
};

impl Paths {
    pub fn new() -> anyhow::Result<Paths> {
        let support = dirs::data_dir()
            .context("no Application Support directory")?
            .join(FOLDER);
        let cache = dirs::cache_dir()
            .context("no Caches directory")?
            .join(FOLDER);
        for dir in [&support, &cache] {
            std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        Ok(Paths { support, cache })
    }

    pub fn database(&self) -> PathBuf {
        self.support.join("knoa.db")
    }

    pub fn settings(&self) -> PathBuf {
        self.support.join("settings.json")
    }

    pub fn dev_credentials(&self) -> PathBuf {
        self.support.join("dev-credentials.json")
    }

    pub fn thumbnails(&self) -> PathBuf {
        self.cache.join("thumbnails")
    }
}
