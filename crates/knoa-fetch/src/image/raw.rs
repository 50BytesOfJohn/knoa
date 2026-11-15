//! A link straight to an image file: the image is the bookmark. From one download and one
//! decode, Knoa keeps a display copy and makes a smaller copy for the AI to look at.

use image::DynamicImage;
use knoa_core::AnalysisImage;
use url::Url;

use super::{
    Preview,
    decode::{self, Decoded, Problem},
    encode,
    facts::Facts,
    scale,
};

/// Longest side of the stored copy: sharp in the detail panel, and a fraction of what a
/// wallpaper costs GPUI to decode, the server to store and sync to move.
const DISPLAY_SIDE: u32 = 1600;
const DISPLAY_QUALITY: u8 = 85;
/// A file within `DISPLAY_SIDE` and at most this big is stored as it is.
const KEEP_AS_IS_BYTES: usize = 1024 * 1024;
/// What the AI sees. At "high" detail this cost about as much as a text page in the
/// research, against up to ten times that for an original.
const ANALYSIS_SIDE: u32 = 1024;
const ANALYSIS_QUALITY: u8 = 80;

/// An image file, read.
pub struct RawImage {
    pub facts: Facts,
    /// The display copy, which becomes the bookmark's image.
    pub preview: Option<Preview>,
    pub analysis: Option<AnalysisImage>,
    /// Why there are no pixels, when there aren't.
    pub problem: Option<Problem>,
}

/// Reads the image file at `url`. `truncated` says `body` holds only the file's start.
pub fn read(url: &Url, body: Vec<u8>, content_type: &str, truncated: bool) -> RawImage {
    let mut facts = Facts::read(&body, content_type, !truncated);
    let decoded = if truncated {
        Err(Problem::TooLargeToDownload)
    } else {
        decode_body(&body, &facts)
    };
    let decoded = match decoded {
        Ok(decoded) => decoded,
        Err(problem) => {
            return RawImage {
                facts,
                preview: None,
                analysis: None,
                problem: Some(problem),
            };
        }
    };
    facts.animated = decoded.animated;
    if decoded.turned {
        (facts.width, facts.height) = (facts.height, facts.width);
    }
    let (preview, analysis) = copies(url, body, &facts, decoded);
    RawImage {
        facts,
        preview,
        analysis,
        problem: None,
    }
}

fn decode_body(body: &[u8], facts: &Facts) -> Result<Decoded, Problem> {
    let codec = facts.codec.ok_or(Problem::Unsupported)?;
    let size = facts.width.zip(facts.height).ok_or(Problem::Damaged)?;
    decode::decode(body, codec, size, DISPLAY_SIDE)
}

fn copies(
    url: &Url,
    body: Vec<u8>,
    facts: &Facts,
    decoded: Decoded,
) -> (Option<Preview>, Option<AnalysisImage>) {
    // Within the display size a JPEG decodes unscaled, so the decode's size is the file's.
    let (width, height) = (decoded.image.width(), decoded.image.height());
    let keep_as_is =
        !decoded.reoriented && body.len() <= KEEP_AS_IS_BYTES && width.max(height) <= DISPLAY_SIDE;
    let (preview, display) = if keep_as_is {
        let preview = facts.codec.map(|codec| Preview {
            url: url.clone(),
            width,
            height,
            extension: codec.extension(),
            bytes: body,
        });
        (preview, decoded.image)
    } else {
        drop(body);
        let display = scale::shrunk(&decoded.image, DISPLAY_SIDE).unwrap_or(decoded.image);
        (display_copy(url, &display), display)
    };
    (preview, analysis_copy(&display))
}

/// Re-encoded: a PNG while it has see-through parts, which only a PNG keeps right on both
/// light and dark cards, otherwise a JPEG.
fn display_copy(url: &Url, image: &DynamicImage) -> Option<Preview> {
    let (bytes, extension) = if encode::has_transparency(image) {
        (encode::png(image)?, "png")
    } else {
        (encode::jpeg(image, DISPLAY_QUALITY)?, "jpg")
    };
    Some(Preview {
        url: url.clone(),
        width: image.width(),
        height: image.height(),
        extension,
        bytes,
    })
}

fn analysis_copy(display: &DynamicImage) -> Option<AnalysisImage> {
    let small = scale::shrunk(display, ANALYSIS_SIDE);
    let image = small.as_ref().unwrap_or(display);
    Some(AnalysisImage {
        media_type: "image/jpeg",
        bytes: encode::jpeg(image, ANALYSIS_QUALITY)?,
        width: image.width(),
        height: image.height(),
    })
}

#[cfg(test)]
mod tests {
    use image::{Rgb, RgbImage};

    use super::*;

    fn url() -> Url {
        Url::parse("https://example.com/wallpaper.jpg").expect("valid URL")
    }

    #[test]
    fn a_large_image_gets_a_display_copy_and_a_smaller_copy_for_the_ai() {
        let photo = RgbImage::from_fn(2000, 1000, |x, _| Rgb([(x % 256) as u8, 90, 160]));
        let jpeg = encode::jpeg(&photo.into(), 90).expect("encodable");

        let image = read(&url(), jpeg, "image/jpeg", false);

        assert_eq!(image.problem, None);
        assert_eq!(
            (image.facts.width, image.facts.height),
            (Some(2000), Some(1000))
        );
        let preview = image.preview.expect("a display copy");
        assert_eq!(
            (preview.width, preview.height, preview.extension),
            (1600, 800, "jpg")
        );
        let analysis = image.analysis.expect("a copy for the AI");
        assert_eq!((analysis.width, analysis.height), (1024, 512));
    }

    #[test]
    fn an_image_over_the_pixel_budget_is_described_without_decoding() {
        // A PNG's signature and header, claiming 5000×5000 pixels.
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        png.extend(5000_u32.to_be_bytes());
        png.extend(5000_u32.to_be_bytes());
        png.extend([8, 6, 0, 0, 0, 0, 0, 0, 0]);

        let image = read(&url(), png, "image/png", false);

        assert_eq!(
            image.problem,
            Some(Problem::TooManyPixels {
                width: 5000,
                height: 5000
            })
        );
        assert_eq!(image.facts.resolution().as_deref(), Some("5000×5000"));
        assert!(image.preview.is_none() && image.analysis.is_none());
    }
}
