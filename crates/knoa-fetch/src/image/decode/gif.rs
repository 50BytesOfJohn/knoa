//! GIFs: the first frame, and whether a second one follows.

use std::io::Cursor;

use image::{AnimationDecoder as _, DynamicImage, ImageDecoder as _, codecs::gif::GifDecoder};

use super::{Decoded, Problem, limits, still};

pub fn decode(bytes: &[u8], too_large: Problem) -> Result<Decoded, Problem> {
    let mut decoder = GifDecoder::new(Cursor::new(bytes)).map_err(|_| Problem::Damaged)?;
    decoder.set_limits(limits()).map_err(|_| Problem::Damaged)?;
    let mut frames = decoder.into_frames();
    let first = frames.next().ok_or(Problem::Damaged)?;
    let first = first.map(|frame| DynamicImage::from(frame.into_buffer()));
    let animated = frames.next().is_some_and(|frame| frame.is_ok());
    still(first, Some(animated), too_large)
}
