//! PNGs. An animated PNG decodes to its first frame.

use std::io::Cursor;

use image::{DynamicImage, ImageDecoder as _, codecs::png::PngDecoder};

use super::{Decoded, Problem, limits, still};

pub fn decode(bytes: &[u8], too_large: Problem) -> Result<Decoded, Problem> {
    let mut decoder = PngDecoder::new(Cursor::new(bytes)).map_err(|_| Problem::Damaged)?;
    // Only worth a mention when it is: few PNGs are animated.
    let animated = decoder.is_apng().ok().filter(|apng| *apng);
    decoder.set_limits(limits()).map_err(|_| Problem::Damaged)?;
    still(DynamicImage::from_decoder(decoder), animated, too_large)
}
