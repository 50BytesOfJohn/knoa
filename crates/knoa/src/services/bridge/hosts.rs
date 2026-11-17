//! Registers `knoa-bridge` with each installed browser by writing its host manifest.
//!
//! Every Chromium browser reads manifests from its own user data directory; Firefox and
//! its forks share Mozilla's. Writing per user needs no admin rights.

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use knoa_paths::Channel;
use serde_json::json;

/// The extension's id in the Chrome Web Store, fixed by the `key` in its manifest.
const CHROME_ORIGIN: &str = "chrome-extension://knagaidgilodmbpjpfejdmmjkopkcbcm/";
/// Firefox identifies the extension by the id in its manifest.
const FIREFOX_EXTENSION_ID: &str = "extension@knoa.dev";

/// User data directories under `~/Library/Application Support`.
const CHROMIUM_DIRS: [&str; 9] = [
    "Google/Chrome",
    "Google/Chrome Beta",
    "Google/Chrome Canary",
    "Chromium",
    "BraveSoftware/Brave-Browser",
    "Microsoft Edge",
    "Vivaldi",
    "net.imput.helium",
    "Arc/User Data",
];
const MOZILLA_DIR: &str = "Mozilla";

/// Writes a host manifest pointing at the bundled bridge for every installed browser.
pub fn register() -> anyhow::Result<()> {
    let Some(bridge) = bundled_bridge()? else {
        tracing::debug!("not running from Knoa.app, so the browser extension isn't registered");
        return Ok(());
    };
    let support = dirs::data_dir().context("no Application Support directory")?;
    let host = Channel::current().bridge_host();
    let path = bridge.to_str().context("the bridge's path is not UTF-8")?;
    let chromium = json!({
        "name": host,
        "description": "Knoa",
        "path": path,
        "type": "stdio",
        "allowed_origins": [CHROME_ORIGIN],
    });
    let mozilla = json!({
        "name": host,
        "description": "Knoa",
        "path": path,
        "type": "stdio",
        "allowed_extensions": [FIREFOX_EXTENSION_ID],
    });
    let targets = CHROMIUM_DIRS
        .iter()
        .map(|dir| (*dir, &chromium))
        .chain([(MOZILLA_DIR, &mozilla)]);
    for (dir, manifest) in targets {
        let browser = support.join(dir);
        if browser.is_dir() {
            let file = browser
                .join("NativeMessagingHosts")
                .join(format!("{host}.json"));
            write_if_changed(&file, &serde_json::to_vec_pretty(manifest)?)?;
        }
    }
    Ok(())
}

/// `Contents/MacOS/knoa-bridge` next to the app's executable, when the app runs from a
/// bundle. A plain `cargo run` has no bundle, and nothing to point the browsers at.
fn bundled_bridge() -> anyhow::Result<Option<PathBuf>> {
    let exe = std::env::current_exe().context("finding Knoa's executable")?;
    let Some(macos) = exe.parent().filter(|dir| dir.ends_with("Contents/MacOS")) else {
        return Ok(None);
    };
    let bridge = macos.join("knoa-bridge");
    anyhow::ensure!(bridge.is_file(), "{} is missing", bridge.display());
    Ok(Some(bridge))
}

/// Browsers read the manifest on every connection; rewriting an unchanged one would only
/// touch the file.
fn write_if_changed(file: &Path, contents: &[u8]) -> anyhow::Result<()> {
    if std::fs::read(file).is_ok_and(|old| old == contents) {
        return Ok(());
    }
    let dir = file.parent().context("a manifest has a directory")?;
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    std::fs::write(file, contents).with_context(|| format!("writing {}", file.display()))?;
    tracing::info!(?file, "registered the browser extension's host");
    Ok(())
}
