//! Where Knoa keeps its files.

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use gpui::Global;

use crate::channel::Channel;

#[derive(Debug, Clone)]
pub struct Paths {
    /// `~/Library/Application Support/Knoa`, or `Knoa Dev` for dev builds: the database,
    /// settings and dev credentials.
    pub support: PathBuf,
    /// `~/Library/Caches/Knoa` (or `Knoa Dev`): thumbnails, safe to delete.
    pub cache: PathBuf,
}

impl Global for Paths {}

impl Paths {
    pub fn new() -> anyhow::Result<Paths> {
        let paths = Self::select(
            Channel::current(),
            &dirs::data_dir().context("no Application Support directory")?,
            &dirs::cache_dir().context("no Caches directory")?,
            std::env::var_os("KNOA_DATA_DIR").as_deref().map(Path::new),
        );
        for dir in [&paths.support, &paths.cache] {
            std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        Ok(paths)
    }

    fn select(channel: Channel, support: &Path, cache: &Path, data_dir: Option<&Path>) -> Self {
        if let (Channel::Dev, Some(root)) = (channel, data_dir) {
            return Self {
                support: root.join("support"),
                cache: root.join("cache"),
            };
        }
        Self {
            support: support.join(channel.name()),
            cache: cache.join(channel.name()),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_paths_never_share_the_release_library() {
        let support = Path::new("/home/test/Library/Application Support");
        let cache = Path::new("/home/test/Library/Caches");
        let scratch = Path::new("/scratch");
        let dev = Paths::select(Channel::Dev, support, cache, None);
        assert_eq!(dev.support, support.join("Knoa Dev"));
        assert_eq!(dev.cache, cache.join("Knoa Dev"));

        let isolated = Paths::select(Channel::Dev, support, cache, Some(scratch));
        assert_eq!(isolated.support, scratch.join("support"));
        assert_eq!(isolated.cache, scratch.join("cache"));

        for override_dir in [None, Some(scratch)] {
            let release = Paths::select(Channel::Release, support, cache, override_dir);
            assert_eq!(release.support, support.join("Knoa"));
            assert_eq!(release.cache, cache.join("Knoa"));
        }
    }
}
