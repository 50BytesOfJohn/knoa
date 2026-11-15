//! WebP files. An animated WebP decodes to its first frame.

use std::io::Cursor;

use image::{DynamicImage, ImageDecoder as _, codecs::webp::WebPDecoder};

use super::{Decoded, Problem, limits, still};

pub fn decode(bytes: &[u8], too_large: Problem) -> Result<Decoded, Problem> {
    let mut decoder = WebPDecoder::new(Cursor::new(bytes)).map_err(|_| Problem::Damaged)?;
    let animated = Some(decoder.has_animation());
    decoder.set_limits(limits()).map_err(|_| Problem::Damaged)?;
    still(DynamicImage::from_decoder(decoder), animated, too_large)
}
