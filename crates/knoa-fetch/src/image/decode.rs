//! Decoding within a memory budget, bounded before it starts. The cloud organizer runs this
//! in a 128 MB Worker next to a body of up to 24 MiB; the Mac allows much larger images.

mod budget;
mod gif;
mod jpeg;
mod png;
mod webp;

use std::fmt;

use image::{DynamicImage, metadata::Orientation};

use super::facts::Codec;
use budget::{JPEG_PIXELS, MAX_ALLOC, OTHER_PIXELS};

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
        Codec::Jpeg => jpeg::decode(bytes, target_side, too_large),
        _ if pixels > OTHER_PIXELS => Err(too_large),
        Codec::Png => png::decode(bytes, too_large),
        Codec::Webp => webp::decode(bytes, too_large),
        Codec::Gif => gif::decode(bytes, too_large),
    }
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

fn limits() -> image::Limits {
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(MAX_ALLOC);
    limits
}
