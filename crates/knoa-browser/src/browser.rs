//! Which browsers Knoa knows, and their profiles on this Mac.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Browsers that keep bookmarks in Chromium's `Bookmarks` JSON file.
///
/// Arc is left out on purpose: its sidebar lives in `StorableSidebar.json`, and its
/// `Bookmarks` file stays almost empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Browser {
    Helium,
    Chrome,
    Brave,
    Edge,
    Vivaldi,
    Chromium,
}

impl Browser {
    pub const ALL: [Browser; 6] = [
        Browser::Helium,
        Browser::Chrome,
        Browser::Brave,
        Browser::Edge,
        Browser::Vivaldi,
        Browser::Chromium,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Browser::Helium => "helium",
            Browser::Chrome => "chrome",
            Browser::Brave => "brave",
            Browser::Edge => "edge",
            Browser::Vivaldi => "vivaldi",
            Browser::Chromium => "chromium",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Browser> {
        Browser::ALL.into_iter().find(|b| b.slug() == slug)
    }

    pub fn name(self) -> &'static str {
        match self {
            Browser::Helium => "Helium",
            Browser::Chrome => "Google Chrome",
            Browser::Brave => "Brave",
            Browser::Edge => "Microsoft Edge",
            Browser::Vivaldi => "Vivaldi",
            Browser::Chromium => "Chromium",
        }
    }

    /// The macOS bundle identifier, which `open -b` launches by.
    pub(crate) fn bundle_id(self) -> &'static str {
        match self {
            Browser::Helium => "net.imput.helium",
            Browser::Chrome => "com.google.Chrome",
            Browser::Brave => "com.brave.Browser",
            Browser::Edge => "com.microsoft.edgemac",
            Browser::Vivaldi => "com.vivaldi.Vivaldi",
            Browser::Chromium => "org.chromium.Chromium",
        }
    }

    /// The full path of [`Browser::user_data_dir`].
    pub(crate) fn user_data_path(self) -> Option<PathBuf> {
        Some(dirs::data_dir()?.join(self.user_data_dir()))
    }

    /// The browser's user data directory, under `~/Library/Application Support`.
    fn user_data_dir(self) -> &'static str {
        match self {
            Browser::Helium => "net.imput.helium",
            Browser::Chrome => "Google/Chrome",
            Browser::Brave => "BraveSoftware/Brave-Browser",
            Browser::Edge => "Microsoft Edge",
            Browser::Vivaldi => "Vivaldi",
            Browser::Chromium => "Chromium",
        }
    }
}

/// A browser profile, as stored in settings: `helium:Default`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileId {
    pub browser: Browser,
    /// The profile's directory name: `Default`, `Profile 1`.
    pub dir: String,
}

impl ProfileId {
    pub fn parse(text: &str) -> Option<ProfileId> {
        let (browser, dir) = text.split_once(':')?;
        Some(ProfileId {
            browser: Browser::from_slug(browser)?,
            dir: dir.to_owned(),
        })
    }
}

impl std::fmt::Display for ProfileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.browser.slug(), self.dir)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub id: ProfileId,
    /// The name the user gave the profile in the browser.
    pub name: String,
    pub bookmarks_path: PathBuf,
}

/// Every profile with a bookmarks file, for every known browser installed on this Mac.
pub fn discover_profiles() -> Vec<Profile> {
    let Some(support) = dirs::data_dir() else {
        return Vec::new();
    };
    Browser::ALL
        .into_iter()
        .flat_map(|browser| profiles_in(browser, &support.join(browser.user_data_dir())))
        .collect()
}

/// Installed browsers whose data macOS won't let Knoa read.
///
/// Brave opts its profile folder into macOS app data protection, so reading it fails with
/// `EPERM` until the user gives Knoa Full Disk Access. Without this the browser would
/// just be missing from every list.
pub fn blocked_browsers() -> Vec<Browser> {
    let Some(support) = dirs::data_dir() else {
        return Vec::new();
    };
    Browser::ALL
        .into_iter()
        .filter(|browser| {
            std::fs::read_dir(support.join(browser.user_data_dir()))
                .is_err_and(|err| err.kind() == std::io::ErrorKind::PermissionDenied)
        })
        .collect()
}

/// The System Settings pane where the user grants Full Disk Access.
pub const FULL_DISK_ACCESS_SETTINGS: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles";

fn profiles_in(browser: Browser, user_data: &Path) -> Vec<Profile> {
    let names = profile_names(user_data);
    let entries = match std::fs::read_dir(user_data) {
        Ok(entries) => entries,
        Err(err) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(?user_data, "can't read {}: {err}", browser.name());
            }
            return Vec::new();
        }
    };
    let mut profiles: Vec<Profile> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let bookmarks_path = entry.path().join("Bookmarks");
            if !bookmarks_path.is_file() {
                return None;
            }
            let dir = entry.file_name().to_string_lossy().into_owned();
            let name = names
                .iter()
                .find(|(d, _)| *d == dir)
                .map_or_else(|| dir.clone(), |(_, n)| n.clone());
            Some(Profile {
                id: ProfileId { browser, dir },
                name,
                bookmarks_path,
            })
        })
        .collect();
    // `Default` first, then the order the browser created them in.
    profiles.sort_by_key(|p| (p.id.dir != "Default", p.id.dir.clone()));
    profiles
}

#[derive(Deserialize)]
struct LocalState {
    profile: ProfileSection,
}

#[derive(Deserialize)]
struct ProfileSection {
    info_cache: std::collections::HashMap<String, ProfileInfo>,
}

#[derive(Deserialize)]
struct ProfileInfo {
    name: String,
}

/// Profile display names from the browser's `Local State` file.
fn profile_names(user_data: &Path) -> Vec<(String, String)> {
    let Ok(text) = std::fs::read_to_string(user_data.join("Local State")) else {
        return Vec::new();
    };
    match serde_json::from_str::<LocalState>(&text) {
        Ok(state) => state
            .profile
            .info_cache
            .into_iter()
            .map(|(dir, info)| (dir, info.name))
            .collect(),
        Err(err) => {
            tracing::debug!(?user_data, "unreadable Local State: {err}");
            Vec::new()
        }
    }
}
