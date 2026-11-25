//! The Worker's memory ceiling, measured on the host with photos as large as those that
//! once took the cloud organizer down, generated rather than stored.

use image::{
    Delay, Frame, ImageEncoder as _, RgbImage, Rgba, RgbaImage,
    codecs::{gif::GifEncoder, png::PngEncoder},
};

use super::*;
use crate::{heap, image::decode::WORKER};

/// A bookmarked file, and the display copy it should get.
struct Case {
    name: &'static str,
    body: Vec<u8>,
    content_type: &'static str,
    display: (u32, u32),
}

fn cases() -> Vec<Case> {
    let case = |name, body, content_type, display| Case {
        name,
        body,
        content_type,
        display,
    };
    vec![
        // The Earth seen from Apollo 17, decoded at half size, then turned upright by its
        // EXIF orientation.
        case(
            "3000×3002 JPEG, turned",
            turned(&photo_jpeg(3000, 3002, 90)),
            "image/jpeg",
            (1501, 1500),
        ),
        // FullMoon2010.
        case(
            "2580×2452 JPEG",
            photo_jpeg(2580, 2452, 90),
            "image/jpeg",
            (1290, 1226),
        ),
        // Fronalpstock: a 15 MB panorama, decoded at a quarter of its size.
        case(
            "10109×4542 JPEG",
            photo_jpeg(10109, 4542, 85),
            "image/jpeg",
            (1600, 719),
        ),
        // A phone's 12 MP photo saved as a PNG, never whole in memory.
        case(
            "4000×3000 PNG",
            photo_png(4000, 3000),
            "image/png",
            (1600, 1200),
        ),
        case(
            "1500×1500 animated GIF",
            animated_gif(1500),
            "image/gif",
            (1500, 1500),
        ),
    ]
}

#[test]
fn image_jobs_stay_within_the_worker_budget() {
    if !heap::alone("image::raw::worker_memory::image_jobs_stay_within_the_worker_budget") {
        return;
    }
    let url = Url::parse("https://example.com/photo").expect("valid URL");
    for Case {
        name,
        body,
        content_type,
        display,
    } in cases()
    {
        // The body was allocated before the measuring began.
        let length = body.len() as u64;
        let (peak, image) = heap::peak(|| read_within(&url, body, content_type, false, WORKER));

        let held = length + peak as u64;
        assert!(held <= WORKER.peak, "{name} held {held} bytes");
        assert_eq!(image.problem, None, "{name}");
        let preview = image.preview.expect("a display copy");
        assert_eq!((preview.width, preview.height), display, "{name}");
        let analysis = image.analysis.expect("a copy for the AI");
        assert_eq!(analysis.width.max(analysis.height), 1024, "{name}");
    }
}

/// Grain of `grain` levels, which gives a file about a photo's size. Rows are copied from
/// one long run of grain at shifting offsets, which keeps an unoptimized test build quick.
fn photo(width: u32, height: u32, grain: u32) -> RgbImage {
    let mut seed = 0x9e37_79b9_u32;
    let noise: Vec<u8> = (0..1 << 17)
        .map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed % grain) as u8
        })
        .collect();
    let row_len = width as usize * 3;
    let mut pixels = Vec::with_capacity(row_len * height as usize);
    for y in 0..height as usize {
        let start = y * 7919 % (noise.len() - row_len);
        pixels.extend_from_slice(&noise[start..start + row_len]);
    }
    RgbImage::from_raw(width, height, pixels).expect("every row")
}

fn photo_jpeg(width: u32, height: u32, quality: u8) -> Vec<u8> {
    encode::jpeg(&photo(width, height, 32).into(), quality).expect("encodable")
}

fn photo_png(width: u32, height: u32) -> Vec<u8> {
    let pixels = photo(width, height, 3);
    let mut file = Vec::new();
    PngEncoder::new(&mut file)
        .write_image(&pixels, width, height, image::ExtendedColorType::Rgb8)
        .expect("encodable");
    file
}

fn animated_gif(side: u32) -> Vec<u8> {
    let frame = |shade: u8| {
        let pixels = RgbaImage::from_fn(side, side, |x, y| {
            Rgba([shade, (x / 8) as u8, (y / 8) as u8, 255])
        });
        Frame::from_parts(pixels, 0, 0, Delay::from_numer_denom_ms(100, 1))
    };
    let mut file = Vec::new();
    GifEncoder::new_with_speed(&mut file, 30)
        .encode_frames([frame(20), frame(220)])
        .expect("encodable");
    file
}

/// `jpeg` with an EXIF segment saying to turn it a quarter turn clockwise.
fn turned(jpeg: &[u8]) -> Vec<u8> {
    let mut tiff = b"MM\0\x2a\0\0\0\x08\0\x01".to_vec();
    // The orientation tag, a short, one of them: 6.
    tiff.extend([0x01, 0x12, 0, 3, 0, 0, 0, 1, 0, 6, 0, 0, 0, 0, 0, 0]);
    let mut segment = vec![0xff, 0xe1];
    segment.extend(((2 + 6 + tiff.len()) as u16).to_be_bytes());
    segment.extend(b"Exif\0\0");
    segment.extend(tiff);
    [&jpeg[..2], &segment, &jpeg[2..]].concat()
}
