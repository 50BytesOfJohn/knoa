//! A link straight to an image file: the image is the bookmark. From one download and one
//! decode, Knoa keeps a display copy and makes a smaller copy for the AI to look at.

use image::DynamicImage;
use knoa_core::AnalysisImage;
use url::Url;

use super::{
    Preview,
    decode::{self, Budget, Decoded, Problem},
    encode,
    facts::Facts,
    scale,
};

/// Longest side of the stored copy: sharp in the detail panel, and a fraction of what a
/// wallpaper costs GPUI to decode, the server to store and sync to move.
const DISPLAY_SIDE: u32 = 1600;
const DISPLAY_QUALITY: u8 = 85;
/// A still, upright file within `DISPLAY_SIDE` and at most this big is stored as it is.
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
    read_within(url, body, content_type, truncated, decode::BUDGET)
}

fn read_within(
    url: &Url,
    body: Vec<u8>,
    content_type: &str,
    truncated: bool,
    budget: Budget,
) -> RawImage {
    let mut facts = Facts::read(&body, content_type, !truncated);
    let decoded = if truncated {
        Err(Problem::TooLargeToDownload)
    } else {
        decode_body(&body, &facts, budget)
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
    if decoded.turned() {
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

fn decode_body(body: &[u8], facts: &Facts, budget: Budget) -> Result<Decoded, Problem> {
    let codec = facts.codec.ok_or(Problem::Unsupported)?;
    let size = facts.width.zip(facts.height).ok_or(Problem::Damaged)?;
    decode::decode(body, codec, size, DISPLAY_SIDE, budget)
}

fn copies(
    url: &Url,
    body: Vec<u8>,
    facts: &Facts,
    decoded: Decoded,
) -> (Option<Preview>, Option<AnalysisImage>) {
    // Dropped unless kept: shrinking is the pipeline's second peak, and the budget leaves
    // the body out of it.
    let kept = keeps_file(body.len(), facts, &decoded).then_some(body);
    let display = decoded.upright(DISPLAY_SIDE);
    let preview = match (kept, facts.codec) {
        (Some(bytes), Some(codec)) => Some(Preview {
            url: url.clone(),
            width: display.width(),
            height: display.height(),
            extension: codec.extension(),
            bytes,
        }),
        _ => display_copy(url, &display),
    };
    (preview, analysis_copy(display))
}

/// Whether the file itself is a fine display copy. Its own size decides, not the decode's: a
/// JPEG decodes scaled down. An animation is always stored as its first frame, since GPUI
/// would decode and keep every frame of it.
fn keeps_file(length: usize, facts: &Facts, decoded: &Decoded) -> bool {
    let fits = facts
        .width
        .zip(facts.height)
        .is_some_and(|(width, height)| width.max(height) <= DISPLAY_SIDE);
    fits && length <= KEEP_AS_IS_BYTES && !decoded.reoriented() && decoded.animated != Some(true)
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

fn analysis_copy(display: DynamicImage) -> Option<AnalysisImage> {
    let image = scale::shrink(display, ANALYSIS_SIDE);
    Some(AnalysisImage {
        media_type: "image/jpeg",
        bytes: encode::jpeg(&image, ANALYSIS_QUALITY)?,
        width: image.width(),
        height: image.height(),
    })
}

#[cfg(test)]
mod worker_memory;

#[cfg(test)]
mod tests {
    use image::{Delay, Frame, Rgb, RgbImage, Rgba, RgbaImage, codecs::gif::GifEncoder};

    use super::*;

    fn url() -> Url {
        Url::parse("https://example.com/wallpaper.jpg").expect("valid URL")
    }

    /// The stored copy's real size, decoded from its bytes.
    fn stored_size(preview: &Preview) -> (u32, u32) {
        let image = ::image::load_from_memory(&preview.bytes).expect("a decodable copy");
        (image.width(), image.height())
    }

    fn gradient_jpeg(width: u32, height: u32) -> Vec<u8> {
        let photo = RgbImage::from_fn(width, height, |x, _| Rgb([(x % 256) as u8, 90, 160]));
        encode::jpeg(&photo.into(), 90).expect("encodable")
    }

    #[test]
    fn a_large_image_gets_a_display_copy_and_a_smaller_copy_for_the_ai() {
        let image = read(&url(), gradient_jpeg(2000, 1000), "image/jpeg", false);

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
        assert_eq!(stored_size(&preview), (1600, 800));
        let analysis = image.analysis.expect("a copy for the AI");
        assert_eq!((analysis.width, analysis.height), (1024, 512));
    }

    #[test]
    fn a_small_file_of_a_jpeg_decoded_scaled_down_is_not_kept_as_it_is() {
        // Decoded at half size in the DCT, it fits the display size; the file doesn't.
        let jpeg = gradient_jpeg(3200, 1800);
        assert!(jpeg.len() <= KEEP_AS_IS_BYTES);

        let image = read(&url(), jpeg, "image/jpeg", false);

        let preview = image.preview.expect("a display copy");
        assert_eq!((preview.width, preview.height), (1600, 900));
        assert_eq!(stored_size(&preview), (1600, 900));
    }

    #[test]
    fn an_animated_gif_is_stored_as_its_first_frame() {
        let frame = |shade: u8| {
            Frame::from_parts(
                RgbaImage::from_pixel(60, 40, Rgba([shade, 30, 30, 255])),
                0,
                0,
                Delay::from_numer_denom_ms(100, 1),
            )
        };
        let mut gif = Vec::new();
        GifEncoder::new(&mut gif)
            .encode_frames([frame(20), frame(220)])
            .expect("encodable");

        let image = read(&url(), gif, "image/gif", false);

        assert_eq!(image.facts.animated, Some(true));
        let preview = image.preview.expect("a display copy");
        assert_eq!(preview.extension, "jpg");
        let first = ::image::load_from_memory(&preview.bytes).expect("decodable");
        assert!(
            first.to_rgb8().get_pixel(0, 0)[0] < 60,
            "the first frame's shade"
        );
    }

    #[test]
    fn a_sliver_is_shrunk_along_its_length_on_the_desktop() {
        let png = worker_memory::sliver_png(1, 2_000_000);

        let image = read_within(&url(), png, "image/png", false, decode::DESKTOP);

        assert_eq!(image.problem, None);
        let preview = image.preview.expect("a display copy");
        assert_eq!((preview.width, preview.height), (1, 1600));
    }

    #[test]
    fn an_image_over_the_pixel_budget_is_described_without_decoding() {
        // A PNG's signature and header, claiming 12000×12000 pixels: over the budget on
        // every target.
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        png.extend(12000_u32.to_be_bytes());
        png.extend(12000_u32.to_be_bytes());
        png.extend([8, 6, 0, 0, 0, 0, 0, 0, 0]);

        let image = read(&url(), png, "image/png", false);

        assert_eq!(
            image.problem,
            Some(Problem::TooManyPixels {
                width: 12000,
                height: 12000
            })
        );
        assert_eq!(image.facts.resolution().as_deref(), Some("12000×12000"));
        assert!(image.preview.is_none() && image.analysis.is_none());
    }
}
