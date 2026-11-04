//! Where Knoa keeps its files.

use std::path::PathBuf;

use anyhow::Context as _;

#[derive(Debug, Clone)]
pub struct Paths {
    /// `~/Library/Application Support/Knoa`: the database, settings and dev credentials.
    pub support: PathBuf,
    /// `~/Library/Caches/Knoa`: thumbnails, safe to delete.
    pub cache: PathBuf,
}

impl Paths {
    pub fn new() -> anyhow::Result<Paths> {
        let support = dirs::data_dir()
            .context("no Application Support directory")?
            .join("Knoa");
        let cache = dirs::cache_dir()
            .context("no Caches directory")?
            .join("Knoa");
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
