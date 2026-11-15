//! Decoding within a memory budget, checked against the file's headers before any pixels
//! are allocated. The cloud organizer runs this in a 128 MB Worker; the Mac allows much
//! larger images.

mod budget;
mod gif;
mod jpeg;
mod png;
mod webp;

use std::fmt;

use image::{DynamicImage, metadata::Orientation};

use super::facts::Codec;
use budget::{Allowance, BUDGET, Budget};

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
/// `target_side` where the format allows. An animation decodes to its first frame.
pub fn decode(
    bytes: &[u8],
    codec: Codec,
    size: (u32, u32),
    target_side: u32,
) -> Result<Decoded, Problem> {
    decode_within(bytes, codec, size, target_side, BUDGET)
}

fn decode_within(
    bytes: &[u8],
    codec: Codec,
    (width, height): (u32, u32),
    target_side: u32,
    budget: Budget,
) -> Result<Decoded, Problem> {
    let allowance = Allowance::new(
        budget,
        bytes.len() as u64,
        Problem::TooManyPixels { width, height },
    );
    if u64::from(width) * u64::from(height) > budget.pixels {
        return Err(allowance.too_large);
    }
    match codec {
        Codec::Jpeg => jpeg::decode(bytes, target_side, &allowance),
        Codec::Png => png::decode(bytes, &allowance),
        Codec::Webp => webp::decode(bytes, (width, height), &allowance),
        Codec::Gif => gif::decode(bytes, (width, height), &allowance),
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

/// A decoder that ran into a limit was refused for size; any other failure is damage.
fn refusal(err: &image::ImageError, allowance: &Allowance) -> Problem {
    match err {
        image::ImageError::Limits(_) => allowance.too_large.clone(),
        _ => Problem::Damaged,
    }
}

fn still(
    image: image::ImageResult<DynamicImage>,
    animated: Option<bool>,
    allowance: &Allowance,
) -> Result<Decoded, Problem> {
    let image = image.map_err(|err| refusal(&err, allowance))?;
    Ok(Decoded {
        image,
        animated,
        turned: false,
        reoriented: false,
    })
}
