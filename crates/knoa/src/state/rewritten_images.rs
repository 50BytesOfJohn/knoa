//! Pictures whose file was written again. Thumbnails are cached under their image URL, so
//! a fetch can land new bytes at a path GPUI already decoded, and GPUI keeps drawing what
//! it decoded until told to drop it. Services note the paths here; the main view, which
//! has the window whose atlas holds them, releases them on its next frame.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use gpui::{App, Global};

#[derive(Clone, Default)]
pub struct RewrittenImages(Arc<Mutex<Vec<PathBuf>>>);

impl Global for RewrittenImages {}

impl RewrittenImages {
    pub fn init(cx: &mut App) {
        cx.set_global(RewrittenImages::default());
    }

    pub fn handle(cx: &App) -> RewrittenImages {
        cx.global::<RewrittenImages>().clone()
    }

    /// Notes a picture written where one may have been before.
    pub fn add(&self, path: PathBuf) {
        self.0.lock().expect("lock poisoned").push(path);
    }

    pub fn take(&self) -> Vec<PathBuf> {
        std::mem::take(&mut *self.0.lock().expect("lock poisoned"))
    }
}
