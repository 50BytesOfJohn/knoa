//! Where API keys are kept (ADR 0008): the Keychain in release builds, a plain file in
//! debug builds, because macOS re-prompts for the Keychain on every unsigned rebuild.

use std::{collections::HashMap, path::PathBuf};

use anyhow::Context as _;
use gpui::{App, AppContext as _, Global, Task};

pub enum Credentials {
    Keychain,
    DevFile(PathBuf),
}

impl Global for Credentials {}

/// The Keychain item's account name; the URL identifies the provider.
const ACCOUNT: &str = "api-key";

impl Credentials {
    pub fn init(dev_file: PathBuf, cx: &mut App) {
        let store = if cfg!(debug_assertions) {
            Credentials::DevFile(dev_file)
        } else {
            Credentials::Keychain
        };
        cx.set_global(store);
    }

    pub fn read(url: &str, cx: &App) -> Task<anyhow::Result<Option<String>>> {
        match cx.global::<Credentials>() {
            Credentials::Keychain => {
                let task = cx.read_credentials(url);
                cx.background_spawn(async move {
                    Ok(task
                        .await?
                        .map(|(_, secret)| String::from_utf8_lossy(&secret).into_owned()))
                })
            }
            Credentials::DevFile(path) => {
                let (path, url) = (path.clone(), url.to_owned());
                cx.background_spawn(async move { Ok(read_dev_file(&path)?.remove(&url)) })
            }
        }
    }

    pub fn write(url: &str, secret: &str, cx: &App) -> Task<anyhow::Result<()>> {
        match cx.global::<Credentials>() {
            Credentials::Keychain => cx.write_credentials(url, ACCOUNT, secret.as_bytes()),
            Credentials::DevFile(path) => {
                let (path, url, secret) = (path.clone(), url.to_owned(), secret.to_owned());
                cx.background_spawn(async move {
                    let mut all = read_dev_file(&path)?;
                    all.insert(url, secret);
                    write_dev_file(&path, &all)
                })
            }
        }
    }

    pub fn delete(url: &str, cx: &App) -> Task<anyhow::Result<()>> {
        match cx.global::<Credentials>() {
            Credentials::Keychain => cx.delete_credentials(url),
            Credentials::DevFile(path) => {
                let (path, url) = (path.clone(), url.to_owned());
                cx.background_spawn(async move {
                    let mut all = read_dev_file(&path)?;
                    all.remove(&url);
                    write_dev_file(&path, &all)
                })
            }
        }
    }
}

fn read_dev_file(path: &PathBuf) -> anyhow::Result<HashMap<String, String>> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).context("dev credentials file is invalid"),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(err) => Err(err.into()),
    }
}

fn write_dev_file(path: &PathBuf, all: &HashMap<String, String>) -> anyhow::Result<()> {
    use std::{io::Write as _, os::unix::fs::OpenOptionsExt as _};
    // 0600: the file holds a secret in plain text.
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(&serde_json::to_vec_pretty(all)?)?;
    Ok(())
}
