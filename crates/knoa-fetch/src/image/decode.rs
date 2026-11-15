//! Decoding within a memory budget, bounded before it starts. The cloud organizer runs this
//! in a 128 MB Worker next to a body of up to 24 MiB; the Mac allows much larger images.

use std::{fmt, io::Cursor};

use image::{
    AnimationDecoder as _, DynamicImage, GrayImage, ImageDecoder, Limits, RgbImage,
    codecs::{gif::GifDecoder, png::PngDecoder, webp::WebPDecoder},
    metadata::Orientation,
};
use jpeg_decoder::{CodingProcess, PixelFormat};

use super::{facts::Codec, scale};

/// Pixel budgets, checked against the header before any decoding. The cloud organizer's
/// 128 MB Worker sets them; the Mac has memory to spare and decodes far larger files, but
/// stays bounded so a hostile header can't ask for gigabytes.
#[cfg(target_arch = "wasm32")]
mod budget {
    /// Baseline JPEGs decode a row at a time and are scaled down in the DCT, so even a
    /// 64 MP photo decodes into a few megabytes.
    pub const JPEG_PIXELS: u64 = 64_000_000;
    /// Progressive JPEGs hold every coefficient until the last scan, about 3 bytes a pixel
    /// at full size whatever the scale.
    pub const PROGRESSIVE_JPEG_PIXELS: u64 = 16_000_000;
    /// PNG, WebP and GIF decode at full size, up to 4 bytes a pixel: 64 MB at this budget.
    pub const OTHER_PIXELS: u64 = 16_000_000;
    /// `image`'s own cap on a decoder's allocations: the backstop for 16-bit PNGs, which
    /// take twice the bytes the pixel budget assumes.
    pub const MAX_ALLOC: u64 = 80 * 1024 * 1024;
}

#[cfg(not(target_arch = "wasm32"))]
mod budget {
    /// 100 MP covers 8K and 10K wallpapers and most camera files in every format.
    pub const JPEG_PIXELS: u64 = 100_000_000;
    pub const PROGRESSIVE_JPEG_PIXELS: u64 = 100_000_000;
    pub const OTHER_PIXELS: u64 = 100_000_000;
    /// A 16-bit RGBA PNG at the pixel budget is 800 MB, decoded once and dropped.
    pub const MAX_ALLOC: u64 = 1024 * 1024 * 1024;
}

use budget::{JPEG_PIXELS, MAX_ALLOC, OTHER_PIXELS, PROGRESSIVE_JPEG_PIXELS};

/// Why an image file has no pixels to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    TooLargeToDownload,
    TooManyPixels { width: u32, height: u32 },
    Unsupported,
    Damaged,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Problem::TooLargeToDownload => write!(f, "the file is too large to download"),
            Problem::TooManyPixels { width, height } => {
                write!(f, "at {width}×{height} it is too large to decode")
            }
            Problem::Unsupported => write!(f, "Knoa can't decode this format"),
            Problem::Damaged => write!(f, "the file couldn't be decoded"),
        }
    }
}

pub struct Decoded {
    /// Upright, and possibly smaller than the file (scaled JPEG decoding).
    pub image: DynamicImage,
    pub animated: Option<bool>,
    /// The photo's orientation turned it a quarter turn, so its sides swapped.
    pub turned: bool,
    /// The file's pixels aren't upright as stored, so they can't be shown as they are.
    pub reoriented: bool,
}

/// Decodes `bytes`, `width`×`height` by its header, no larger than needed for
/// `target_side` where the format allows.
pub fn decode(
    bytes: &[u8],
    codec: Codec,
    (width, height): (u32, u32),
    target_side: u32,
) -> Result<Decoded, Problem> {
    let pixels = u64::from(width) * u64::from(height);
    let too_large = Problem::TooManyPixels { width, height };
    match codec {
        Codec::Jpeg if pixels > JPEG_PIXELS => Err(too_large),
        Codec::Jpeg => jpeg(bytes, target_side, too_large),
        _ if pixels > OTHER_PIXELS => Err(too_large),
        Codec::Png => png(bytes, too_large),
        Codec::Webp => webp(bytes, too_large),
        Codec::Gif => gif(bytes, too_large),
    }
}

fn jpeg(bytes: &[u8], target_side: u32, too_large: Problem) -> Result<Decoded, Problem> {
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

fn upright(mut image: DynamicImage, orientation: Orientation) -> Decoded {
    image.apply_orientation(orientation);
    Decoded {
        image,
        animated: None,
        turned: matches!(
            orientation,
            Orientation::Rotate90
                | Orientation::Rotate270
                | Orientation::Rotate90FlipH
                | Orientation::Rotate270FlipH
        ),
        reoriented: orientation != Orientation::NoTransforms,
    }
}

/// An animated PNG decodes to its first frame.
fn png(bytes: &[u8], too_large: Problem) -> Result<Decoded, Problem> {
    let mut decoder = PngDecoder::new(Cursor::new(bytes)).map_err(|_| Problem::Damaged)?;
    // Only worth a mention when it is: few PNGs are animated.
    let animated = decoder.is_apng().ok().filter(|apng| *apng);
    decoder.set_limits(limits()).map_err(|_| Problem::Damaged)?;
    still(DynamicImage::from_decoder(decoder), animated, too_large)
}

/// An animated WebP decodes to its first frame.
fn webp(bytes: &[u8], too_large: Problem) -> Result<Decoded, Problem> {
    let mut decoder = WebPDecoder::new(Cursor::new(bytes)).map_err(|_| Problem::Damaged)?;
    let animated = Some(decoder.has_animation());
    decoder.set_limits(limits()).map_err(|_| Problem::Damaged)?;
    still(DynamicImage::from_decoder(decoder), animated, too_large)
}

/// The first frame; reading whether a second one follows says if the GIF is animated.
fn gif(bytes: &[u8], too_large: Problem) -> Result<Decoded, Problem> {
    let mut decoder = GifDecoder::new(Cursor::new(bytes)).map_err(|_| Problem::Damaged)?;
    decoder.set_limits(limits()).map_err(|_| Problem::Damaged)?;
    let mut frames = decoder.into_frames();
    let first = frames.next().ok_or(Problem::Damaged)?;
    let first = first.map(|frame| DynamicImage::from(frame.into_buffer()));
    let animated = frames.next().is_some_and(|frame| frame.is_ok());
    still(first, Some(animated), too_large)
}

fn still(
    image: image::ImageResult<DynamicImage>,
    animated: Option<bool>,
    too_large: Problem,
) -> Result<Decoded, Problem> {
    let image = image.map_err(|err| match err {
        image::ImageError::Limits(_) => too_large,
        _ => Problem::Damaged,
    })?;
    Ok(Decoded {
        image,
        animated,
        turned: false,
        reoriented: false,
    })
}

fn limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_alloc = Some(MAX_ALLOC);
    limits
}
