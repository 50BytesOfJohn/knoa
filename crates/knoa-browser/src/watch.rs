//! Watching bookmark files for changes.
//!
//! Chromium saves `Bookmarks` by writing a temporary file and renaming it over the old one,
//! so the watch is on the profile directory and filters for that file name. macOS delivers
//! these through `FSEvents`, which costs nothing while nothing changes. Saves come in bursts
//! (the browser also rewrites `Bookmarks.bak`), so changes are debounced.

use std::{
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use notify::{EventKind, RecursiveMode, Watcher as _};

use crate::{Error, Profile, ProfileId};

const DEBOUNCE: Duration = Duration::from_millis(1500);

/// Keeps watching while alive.
pub struct Watcher {
    _inner: notify::RecommendedWatcher,
}

/// Calls `on_change` with the profile whose bookmarks changed, at most once per burst of
/// writes. `on_change` runs on a background thread.
pub fn watch(
    profiles: &[Profile],
    on_change: impl Fn(&ProfileId) + Send + 'static,
) -> Result<Watcher, Error> {
    let (tx, rx) = mpsc::channel::<PathBuf>();
    let mut inner =
        notify::recommended_watcher(move |event: notify::Result<notify::Event>| match event {
            Ok(event) if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) => {
                for path in event.paths {
                    if path.file_name().is_some_and(|name| name == "Bookmarks") {
                        // The receiver only goes away when the watcher is dropped.
                        let _ = tx.send(path);
                    }
                }
            }
            Ok(_) => {}
            Err(err) => tracing::warn!("bookmark watch error: {err}"),
        })?;

    let mut watched = Vec::new();
    for profile in profiles {
        if let Some(dir) = profile.bookmarks_path.parent() {
            inner.watch(dir, RecursiveMode::NonRecursive)?;
            // Events carry canonical paths; compare against the same form.
            let path = profile
                .bookmarks_path
                .canonicalize()
                .unwrap_or_else(|_| profile.bookmarks_path.clone());
            watched.push((path, profile.id.clone()));
        }
    }

    thread::Builder::new()
        .name("knoa-bookmark-watch".into())
        .spawn(move || debounce(&rx, &watched, &on_change))
        .map_err(|err| Error::Io {
            path: PathBuf::new(),
            err,
        })?;
    Ok(Watcher { _inner: inner })
}

fn debounce(
    rx: &mpsc::Receiver<PathBuf>,
    watched: &[(PathBuf, ProfileId)],
    on_change: &impl Fn(&ProfileId),
) {
    while let Ok(first) = rx.recv() {
        let mut changed = vec![first];
        let deadline = Instant::now() + DEBOUNCE;
        while let Ok(path) = rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            changed.push(path);
        }
        for (path, id) in watched {
            if changed.iter().any(|c| c == path) {
                on_change(id);
            }
        }
    }
}
