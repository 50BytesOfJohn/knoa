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

use super::{facts::Codec, scale};
use budget::Allowance;
#[cfg(test)]
pub use budget::WORKER;
pub use budget::{BUDGET, Budget};

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
    /// As the file stores it, not yet turned upright, and possibly smaller than the file:
    /// a JPEG scaled in the DCT, a PNG shrunk as it is read.
    image: DynamicImage,
    orientation: Orientation,
    pub animated: Option<bool>,
}

impl Decoded {
    fn new(image: DynamicImage, orientation: Orientation, animated: Option<bool>) -> Decoded {
        Decoded {
            image,
            orientation,
            animated,
        }
    }

    /// The photo's orientation turns it a quarter turn, so its sides swap.
    pub fn turned(&self) -> bool {
        matches!(
            self.orientation,
            Orientation::Rotate90
                | Orientation::Rotate270
                | Orientation::Rotate90FlipH
                | Orientation::Rotate270FlipH
        )
    }

    /// The file's pixels aren't upright as stored, so they can't be shown as they are.
    pub fn reoriented(&self) -> bool {
        self.orientation != Orientation::NoTransforms
    }

    /// Shrunk to fit `max_side`, then turned upright: a quarter turn copies the pixels, so
    /// it copies the smaller image.
    pub fn upright(self, max_side: u32) -> DynamicImage {
        let mut image = scale::shrink(self.image, max_side);
        image.apply_orientation(self.orientation);
        image
    }
}

/// Decodes `bytes`, `width`×`height` by its header, within `budget`, no larger than
/// needed for `target_side` where the format allows. An animation decodes to its first
/// frame.
pub fn decode(
    bytes: &[u8],
    codec: Codec,
    (width, height): (u32, u32),
    target_side: u32,
    budget: Budget,
) -> Result<Decoded, Problem> {
    let allowance = Allowance::new(
        budget,
        bytes.len() as u64,
        upright_bytes(width, height, target_side),
        Problem::TooManyPixels { width, height },
    );
    if u64::from(width) * u64::from(height) > budget.pixels {
        return Err(allowance.too_large);
    }
    match codec {
        Codec::Jpeg => jpeg::decode(bytes, target_side, &allowance),
        Codec::Png => png::decode(bytes, target_side, &allowance),
        Codec::Webp => webp::decode(bytes, (width, height), &allowance),
        Codec::Gif => gif::decode(bytes, (width, height), &allowance),
    }
}

/// What [`Decoded::upright`] holds beside the decoded pixels, at four channels: the
/// resampler, or for an image that needs no shrinking, a quarter turn's copy.
fn upright_bytes(width: u32, height: u32, max_side: u32) -> u64 {
    let target = scale::fit(width, height, max_side);
    if target == (width, height) {
        u64::from(width) * u64::from(height) * 4
    } else {
        scale::Resampler::bytes((width, height), target, 4)
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
    Ok(Decoded::new(image, Orientation::NoTransforms, animated))
}
