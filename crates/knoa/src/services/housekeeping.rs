//! Cleaning up at launch: bookmarks whose time in Trash is up, and thumbnails no bookmark
//! uses any more.

use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use gpui::{App, Entity};

use crate::state::library::Library;

/// The organizer writes a thumbnail before it saves the path to its bookmark, so a newer
/// file may still be on its way in.
const MIN_AGE: Duration = Duration::from_hours(1);

pub fn run(library: &Entity<Library>, thumbnails: PathBuf, cx: &mut App) {
    let now = jiff::Timestamp::now().as_second();
    library.update(cx, |library, cx| {
        library.change(cx, move |store| {
            let purged = store.purge_expired_trash(now)?;
            let in_use = store.image_paths()?;
            match sweep(&thumbnails, &in_use, SystemTime::now()) {
                Ok(removed) => tracing::info!(purged, removed, "cleaned up Trash and thumbnails"),
                Err(err) => tracing::warn!("can't clean up thumbnails: {err}"),
            }
            Ok(())
        });
    });
}

/// Deletes files in `dir` that aren't in `in_use` and are older than `MIN_AGE`. Returns how
/// many were deleted.
fn sweep(dir: &Path, in_use: &HashSet<PathBuf>, now: SystemTime) -> io::Result<usize> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(err) => return Err(err),
    };
    let mut removed = 0;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let modified = entry.metadata()?.modified()?;
        let old = now.duration_since(modified).is_ok_and(|age| age > MIN_AGE);
        if old && !in_use.contains(&path) {
            std::fs::remove_file(&path)?;
            removed += 1;
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_keeps_thumbnails_in_use_and_new_ones() {
        let dir = std::env::temp_dir().join(format!("knoa-sweep-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let [used, unused, fresh] = ["used.jpg", "unused.jpg", "fresh.jpg"].map(|name| {
            let path = dir.join(name);
            std::fs::write(&path, b"x").unwrap();
            path
        });
        let later = SystemTime::now() + MIN_AGE * 2;
        std::fs::File::options()
            .write(true)
            .open(&fresh)
            .unwrap()
            .set_modified(later)
            .unwrap();

        let removed = sweep(&dir, &HashSet::from([used.clone()]), later).unwrap();

        assert_eq!(removed, 1);
        assert!(used.exists() && fresh.exists() && !unused.exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
