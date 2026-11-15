//! Lets go of pictures that scrolled far away, or whose file was written again. GPUI keeps
//! every image it decodes, in memory and in the GPU's atlas, until it is told to drop it: a
//! 1600 px display copy costs about 11 MB, so scrolling through 300 pictures held 3.8 GB.
//! Releasing them keeps that scroll under 0.9 GB.

use std::{collections::HashSet, path::PathBuf};

use gpui::{Context, ImgResourceLoader, Resource, Window};

use super::OVERSCAN;
use crate::{state::rewritten_images::RewrittenImages, ui::main::MainView};

/// Pictures this far beyond the viewport stay decoded, so scrolling back a little doesn't
/// decode them again. About a screen: much more holds most of a few hundred small tiles.
const KEEP_BEYOND: f32 = 900.;

/// The pictures the cards have shown and GPUI may still hold.
#[derive(Default)]
pub struct DecodedImages(HashSet<PathBuf>);

impl MainView {
    /// Records the pictures of the cards built this frame, and drops those now far from
    /// the band from `top` to `bottom`. The selected bookmark's picture stays: the detail
    /// panel or the viewer may be showing it.
    pub(super) fn release_far_images(
        &mut self,
        top: f32,
        bottom: f32,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let results = &self.layout.results;
        let path_at = |index: usize| results.get(index)?.image.as_ref().map(|i| i.path.clone());
        let cards = &self.layout.layout;
        self.decoded.0.extend(
            cards
                .visible(top - OVERSCAN, bottom + OVERSCAN)
                .filter_map(|card| path_at(card.index)),
        );
        let mut keep: HashSet<PathBuf> = cards
            .visible(top - KEEP_BEYOND, bottom + KEEP_BEYOND)
            .filter_map(|card| path_at(card.index))
            .collect();
        let selected = self
            .selected
            .and_then(|id| self.library.read(cx).bookmark(id))
            .and_then(|b| b.image.as_ref().map(|i| i.path.clone()));
        self.release_rewritten_images(selected.as_ref(), window, cx);
        keep.extend(selected);

        let far: Vec<PathBuf> = self.decoded.0.difference(&keep).cloned().collect();
        for path in far {
            self.decoded.0.remove(&path);
            release(path, window, cx);
        }
    }

    /// Drops pictures whose file a fetch wrote again, so they are decoded anew from it,
    /// the selected one too. Only those the cards or the selection may show were decoded:
    /// asking GPUI for any other would start decoding it.
    fn release_rewritten_images(
        &mut self,
        selected: Option<&PathBuf>,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        for path in RewrittenImages::handle(cx).take() {
            if self.decoded.0.remove(&path) || selected == Some(&path) {
                release(path, window, cx);
            } else {
                cx.remove_asset::<ImgResourceLoader>(&Resource::from(path));
            }
        }
    }
}

/// Drops a picture from GPUI's asset cache and from this window's atlas. A pending load is
/// dropped with it.
fn release(path: PathBuf, window: &mut Window, cx: &mut Context<'_, MainView>) {
    let resource = Resource::from(path);
    if let Some(Ok(image)) = cx.fetch_asset::<ImgResourceLoader>(&resource) {
        cx.drop_image(image, Some(window));
    }
    cx.remove_asset::<ImgResourceLoader>(&resource);
}
