//! Chromium-family browsers (Helium, Chrome, Brave, ...): finding their profiles, reading
//! their `Bookmarks` file and watching it for changes.
//!
//! Sync is one-way. Knoa only ever reads the browser's files. It also opens links in a
//! chosen profile.

mod bookmarks;
mod browser;
mod open;
mod watch;

pub use bookmarks::{BookmarksFile, read_bookmarks};
pub use browser::{
    Browser, FULL_DISK_ACCESS_SETTINGS, Profile, ProfileId, blocked_browsers, discover_profiles,
};
pub use open::{Activation, open_in_default, open_in_profile};
pub use watch::{Watcher, watch};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("can't read {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("{path} is not a bookmarks file: {source}")]
    Parse {
        path: std::path::PathBuf,
        source: serde_json::Error,
    },
    #[error("profile {0} no longer exists")]
    MissingProfile(ProfileId),
    #[error("can't open the link: {0}")]
    Open(std::io::Error),
    #[error("can't watch bookmarks: {0}")]
    Watch(#[from] notify::Error),
}
