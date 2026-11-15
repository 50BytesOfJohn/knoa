//! JPEGs, through jpeg-decoder: it scales down in the DCT, which `image`'s own decoder
//! can't.

use image::{DynamicImage, GrayImage, RgbImage, metadata::Orientation};
use jpeg_decoder::{CodingProcess, PixelFormat};

use super::{Decoded, Problem, budget::PROGRESSIVE_JPEG_PIXELS, upright};
use crate::image::scale;

pub fn decode(bytes: &[u8], target_side: u32, too_large: Problem) -> Result<Decoded, Problem> {
    let mut decoder = jpeg_decoder::Decoder::new(bytes);
    decoder.read_info().map_err(|_| Problem::Damaged)?;
    let info = decoder.info().ok_or(Problem::Damaged)?;
    let (width, height) = (u32::from(info.width), u32::from(info.height));
    if info.coding_process == CodingProcess::DctProgressive
        && u64::from(width) * u64::from(height) > PROGRESSIVE_JPEG_PIXELS
    {
        return Err(too_large);
    }
    let (wanted_width, wanted_height) = scale::fit(width, height, target_side);
    // Both fit: a JPEG's sides are 16-bit.
    decoder
        .scale(wanted_width as u16, wanted_height as u16)
        .map_err(|_| Problem::Damaged)?;
    let pixels = decoder.decode().map_err(|_| Problem::Damaged)?;
    let info = decoder.info().ok_or(Problem::Damaged)?;
    let (width, height) = (u32::from(info.width), u32::from(info.height));
    let image = match info.pixel_format {
        PixelFormat::L8 => GrayImage::from_raw(width, height, pixels).map(DynamicImage::from),
        PixelFormat::RGB24 => RgbImage::from_raw(width, height, pixels).map(DynamicImage::from),
        // Print files: rare on the web, and their colors need a profile to be right.
        PixelFormat::CMYK32 | PixelFormat::L16 => return Err(Problem::Unsupported),
    }
    .ok_or(Problem::Damaged)?;
    let orientation = decoder
        .exif_data()
        .and_then(Orientation::from_exif_chunk)
        .unwrap_or(Orientation::NoTransforms);
    Ok(upright(image, orientation))
}
