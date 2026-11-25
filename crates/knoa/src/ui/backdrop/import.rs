//! Turns a picture the user chose into Knoa's own copy: a JPEG no larger than a big
//! display needs. macOS's `sips` does the work, so every format macOS reads, HEIC
//! wallpapers included, works without a decoder of Knoa's own.

use std::{
    io,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context as _, bail};

const SIPS: &str = "/usr/bin/sips";
/// The longest side of the copy: a 5K display's width, past which nothing shows more.
const MAX_SIDE: u32 = 3200;

const PREFIX: &str = "background-";
const EXTENSION: &str = ".jpg";

/// Copies `source` into `dir` and returns the copy's path. Blocks: run it in the
/// background. Each copy has a new name, so a new picture is never mistaken for the one
/// already shown.
pub fn import(source: &Path, dir: &Path) -> anyhow::Result<PathBuf> {
    let (width, height) = pixel_size(source)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let target = dir.join(format!("{PREFIX}{stamp}{EXTENSION}"));
    let partial = dir.join(format!("{PREFIX}{stamp}.partial{EXTENSION}"));
    let mut sips = Command::new(SIPS);
    sips.args(["-s", "format", "jpeg", "-s", "formatOptions", "85"]);
    if width.max(height) > MAX_SIDE {
        sips.arg("-Z").arg(MAX_SIDE.to_string());
    }
    let output = sips
        .arg(source)
        .arg("--out")
        .arg(&partial)
        .output()
        .context("running sips")?;
    if !output.status.success() {
        let _ = std::fs::remove_file(&partial);
        bail!(
            "sips failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    std::fs::rename(&partial, &target).context("moving the copy into place")?;
    Ok(target)
}

/// Deletes the copies in `dir` other than `in_use`, half-made ones included. Returns how
/// many were deleted. Blocks: run it in the background.
pub fn remove_unused(dir: &Path, in_use: Option<&Path>) -> io::Result<usize> {
    let mut removed = 0;
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let is_copy = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(PREFIX) && name.ends_with(EXTENSION));
        if is_copy && in_use != Some(path.as_path()) {
            std::fs::remove_file(&path)?;
            removed += 1;
        }
    }
    Ok(removed)
}

fn pixel_size(source: &Path) -> anyhow::Result<(u32, u32)> {
    let output = Command::new(SIPS)
        .args(["-g", "pixelWidth", "-g", "pixelHeight"])
        .arg(source)
        .output()
        .context("running sips")?;
    if !output.status.success() {
        bail!("sips can't read {}", source.display());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let value = |key: &str| -> anyhow::Result<u32> {
        text.lines()
            .find_map(|line| line.trim().strip_prefix(key)?.trim().parse().ok())
            .with_context(|| format!("sips gave no {key} for {}", source.display()))
    };
    Ok((value("pixelWidth:")?, value("pixelHeight:")?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_unused_keeps_the_copy_in_use_and_other_files() {
        let dir = std::env::temp_dir().join(format!("knoa-backdrop-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let names = [
            "background-1.jpg",
            "background-2.jpg",
            "background-3.partial.jpg",
            "settings.json",
        ];
        let [used, old, partial, other] = names.map(|name| {
            let path = dir.join(name);
            std::fs::write(&path, b"x").unwrap();
            path
        });

        let removed = remove_unused(&dir, Some(&used)).unwrap();

        assert_eq!(removed, 2);
        assert!(used.exists() && other.exists());
        assert!(!old.exists() && !partial.exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
