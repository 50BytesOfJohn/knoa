//! Opening a link in a chosen browser profile.

use std::process::Command;

use crate::{Error, ProfileId};

/// Whether opening a link brings the browser to the front.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activation {
    Foreground,
    /// The browser opens the link but Knoa stays in front, like a background tab.
    Background,
}

impl Activation {
    fn flags(self) -> &'static [&'static str] {
        match self {
            Activation::Foreground => &[],
            Activation::Background => &["-g"],
        }
    }
}

/// Opens `url` in the default browser.
pub fn open_in_default(url: &str, activation: Activation) -> Result<(), Error> {
    run(Command::new("open").args(activation.flags()).arg(url))
}

/// Opens `url` in the given profile, starting the browser if it isn't running.
///
/// A profile that no longer exists is an error rather than a silent no-op: Chromium would
/// create a fresh, empty profile with that name.
pub fn open_in_profile(
    url: &str,
    profile: &ProfileId,
    activation: Activation,
) -> Result<(), Error> {
    if is_missing(profile) {
        return Err(Error::MissingProfile(profile.clone()));
    }
    // `-n` starts a second process even when the browser is running, so it gets our
    // arguments; Chromium then hands them to the running instance and exits.
    run(Command::new("open")
        .args(activation.flags())
        .args(["-n", "-b", profile.browser.bundle_id(), "--args"])
        .arg(format!("--profile-directory={}", profile.dir))
        .arg(url))
}

fn run(command: &mut Command) -> Result<(), Error> {
    let status = command.status().map_err(Error::Open)?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Open(std::io::Error::other(format!(
            "`open` exited with {status}"
        ))))
    }
}

/// A profile counts as missing only when Knoa can see that it's gone. When macOS blocks
/// the browser's folder (Brave), there's nothing to check, and the browser can still
/// open the profile.
fn is_missing(profile: &ProfileId) -> bool {
    let Some(user_data) = profile.browser.user_data_path() else {
        return false;
    };
    match std::fs::metadata(user_data.join(&profile.dir)) {
        Ok(meta) => !meta.is_dir(),
        Err(err) => err.kind() == std::io::ErrorKind::NotFound,
    }
}
