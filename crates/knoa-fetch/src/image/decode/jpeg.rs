//! JPEGs, through jpeg-decoder: it scales down in the DCT, which `image`'s own decoder
//! can't.

use image::{DynamicImage, GrayImage, RgbImage, metadata::Orientation};
use jpeg_decoder::{CodingProcess, PixelFormat};

use super::{Decoded, Problem, budget::Allowance, upright};
use crate::image::scale;

pub fn decode(bytes: &[u8], target_side: u32, allowance: &Allowance) -> Result<Decoded, Problem> {
    let frame = Frame::read(bytes).ok_or(Problem::Damaged)?;
    let mut decoder = jpeg_decoder::Decoder::new(bytes);
    decoder.read_info().map_err(|_| Problem::Damaged)?;
    let info = decoder.info().ok_or(Problem::Damaged)?;
    let (width, height) = (u32::from(info.width), u32::from(info.height));
    if (width, height) != (frame.width, frame.height) {
        return Err(Problem::Damaged);
    }
    if !supported(info) {
        return Err(Problem::Unsupported);
    }
    let progressive = info.coding_process == CodingProcess::DctProgressive;
    let eighths = frame
        .eighths_within(
            scale::fit(width, height, target_side),
            progressive,
            allowance,
        )
        .ok_or_else(|| allowance.too_large.clone())?;
    let (scaled_width, scaled_height) = frame.scaled(eighths);
    // Both fit: a JPEG's sides are 16-bit.
    let scaled = decoder
        .scale(scaled_width as u16, scaled_height as u16)
        .map_err(|_| Problem::Damaged)?;
    let pixels = decoder.decode().map_err(|_| Problem::Damaged)?;
    let (width, height) = (u32::from(scaled.0), u32::from(scaled.1));
    let image = match info.pixel_format {
        PixelFormat::L8 => GrayImage::from_raw(width, height, pixels).map(DynamicImage::from),
        PixelFormat::RGB24 => RgbImage::from_raw(width, height, pixels).map(DynamicImage::from),
        PixelFormat::CMYK32 => {
            RgbImage::from_raw(width, height, cmyk_to_rgb(pixels)).map(DynamicImage::from)
        }
        PixelFormat::L16 => return Err(Problem::Unsupported),
    }
    .ok_or(Problem::Damaged)?;
    let orientation = decoder
        .exif_data()
        .and_then(Orientation::from_exif_chunk)
        .unwrap_or(Orientation::NoTransforms);
    Ok(upright(image, orientation))
}

/// Lossless JPEGs decode at full size into 16-bit samples whatever the scale, and only
/// they have 16-bit gray. Both are archival and medical files, not web pictures.
fn supported(info: jpeg_decoder::ImageInfo) -> bool {
    info.coding_process != CodingProcess::Lossless && info.pixel_format != PixelFormat::L16
}

/// CMYK in place, as RGB. jpeg-decoder hands out ink amounts, Adobe's inverted storage
/// already undone; without the file's color profile the colors are close, not exact.
fn cmyk_to_rgb(mut pixels: Vec<u8>) -> Vec<u8> {
    let count = pixels.len() / 4;
    for i in 0..count {
        let [c, m, y, k] = [0, 1, 2, 3].map(|channel| u16::from(pixels[i * 4 + channel]));
        let light = |ink: u16| ((255 - ink) * (255 - k) / 255) as u8;
        // The pixel's three bytes land on bytes already read, its own or earlier ones.
        pixels[i * 3] = light(c);
        pixels[i * 3 + 1] = light(m);
        pixels[i * 3 + 2] = light(y);
    }
    pixels.truncate(count * 3);
    pixels
}

/// The frame header: the size and each component's sampling, which jpeg-decoder keeps to
/// itself.
#[derive(Debug, PartialEq, Eq)]
struct Frame {
    width: u32,
    height: u32,
    /// Each component's horizontal and vertical sampling factors.
    sampling: Vec<(u32, u32)>,
}

impl Frame {
    /// Reads the segments up to the first frame header.
    fn read(bytes: &[u8]) -> Option<Frame> {
        if bytes.get(..2)? != [0xff, 0xd8] {
            return None;
        }
        let mut at = 2;
        loop {
            // Fill bytes may pad a marker.
            while *bytes.get(at + 1)? == 0xff {
                at += 1;
            }
            if *bytes.get(at)? != 0xff {
                return None;
            }
            let marker = *bytes.get(at + 1)?;
            if matches!(marker, 0x01 | 0xd0..=0xd7) {
                at += 2;
                continue;
            }
            let length = usize::from(u16::from_be_bytes([
                *bytes.get(at + 2)?,
                *bytes.get(at + 3)?,
            ]));
            let segment = bytes.get(at + 4..(at + 2).checked_add(length)?)?;
            match marker {
                0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf => return parse(segment),
                // A scan or the end before any frame.
                0xda | 0xd9 => return None,
                _ => at += 2 + length,
            }
        }
    }

    /// The DCT scale, in eighths, to decode at: the smallest that reaches `wanted`, or
    /// when that doesn't fit `allowance` the largest smaller one that does. A picture
    /// smaller than wanted beats none.
    fn eighths_within(
        &self,
        (wanted_width, wanted_height): (u32, u32),
        progressive: bool,
        allowance: &Allowance,
    ) -> Option<u32> {
        let reaches = |eighths: u32| {
            let (width, height) = self.scaled(eighths);
            width >= wanted_width || height >= wanted_height
        };
        let preferred = EIGHTHS.into_iter().find(|&e| reaches(e)).unwrap_or(8);
        EIGHTHS
            .into_iter()
            .rev()
            .filter(|&eighths| eighths <= preferred)
            .find(|&eighths| allowance.fits(self.peak_bytes(eighths, progressive)))
    }

    /// The decoded size at a DCT scale of `eighths`, rounded up as jpeg-decoder does.
    fn scaled(&self, eighths: u32) -> (u32, u32) {
        let side = |n: u32| (n * eighths).div_ceil(8);
        (side(self.width), side(self.height))
    }

    /// What decoding at a DCT scale of `eighths` holds at its peak: each component's plane
    /// at that scale, the interleaved pixels (a gray plane is its own) and an EXIF turn's
    /// copy of them. A progressive file also keeps every coefficient, two bytes per sample
    /// at full size, until its last scan.
    fn peak_bytes(&self, eighths: u32, progressive: bool) -> u64 {
        let max_h = self.sampling.iter().map(|s| s.0).max().unwrap_or(1);
        let max_v = self.sampling.iter().map(|s| s.1).max().unwrap_or(1);
        let mcus_across = u64::from(self.width.div_ceil(8 * max_h));
        let mcus_down = u64::from(self.height.div_ceil(8 * max_v));
        let idct = u64::from(eighths);
        let mut bytes = 0;
        for &(h, v) in &self.sampling {
            let blocks = mcus_across * u64::from(h) * mcus_down * u64::from(v);
            bytes += blocks * idct * idct;
            if progressive {
                bytes += blocks * 64 * 2;
            }
        }
        let (scaled_width, scaled_height) = self.scaled(eighths);
        let components = self.sampling.len() as u64;
        let pixels = components * u64::from(scaled_width) * u64::from(scaled_height);
        let interleaved = if components > 1 { pixels } else { 0 };
        bytes + interleaved + pixels
    }
}

fn parse(segment: &[u8]) -> Option<Frame> {
    let height = u32::from(u16::from_be_bytes([*segment.get(1)?, *segment.get(2)?]));
    let width = u32::from(u16::from_be_bytes([*segment.get(3)?, *segment.get(4)?]));
    let count = usize::from(*segment.get(5)?);
    let components = segment.get(6..6 + count * 3)?;
    let sampling = components
        .as_chunks::<3>()
        .0
        .iter()
        .map(|[_, factors, _]| (u32::from(factors >> 4), u32::from(factors & 0x0f)))
        .collect::<Vec<_>>();
    let valid = |n: u32| (1..=4).contains(&n);
    if count == 0 || sampling.iter().any(|&(h, v)| !valid(h) || !valid(v)) {
        return None;
    }
    Some(Frame {
        width,
        height,
        sampling,
    })
}

/// The DCT scales jpeg-decoder offers, in eighths, smallest first.
const EIGHTHS: [u32; 4] = [1, 2, 4, 8];

#[cfg(test)]
mod tests {
    use super::{super::budget::WORKER, *};

    /// What the Worker leaves a decoder beside a body of 600 KB.
    fn worker_allowance() -> Allowance {
        Allowance::new(WORKER, 600_000, Problem::Damaged)
    }

    fn frame(width: u32, height: u32, sampling: &[(u32, u32)]) -> Frame {
        Frame {
            width,
            height,
            sampling: sampling.to_vec(),
        }
    }

    const SAMPLING_420: [(u32, u32); 3] = [(2, 2), (1, 1), (1, 1)];
    const SAMPLING_444: [(u32, u32); 3] = [(1, 1), (1, 1), (1, 1)];

    #[test]
    fn a_progressive_7_mp_photo_decodes_at_full_size_in_the_worker() {
        // wallhaven-6le97q: half size would be 1152×1536, under the display size.
        let frame = frame(2304, 3072, &SAMPLING_420);

        let eighths = frame.eighths_within((1200, 1600), true, &worker_allowance());

        assert_eq!(eighths, Some(8));
    }

    #[test]
    fn a_progressive_photo_too_large_for_its_display_scale_decodes_smaller() {
        let frame = frame(6000, 4000, &SAMPLING_420);

        let eighths = frame.eighths_within((1600, 1067), true, &worker_allowance());

        assert_eq!(
            eighths.map(|eighths| frame.scaled(eighths)),
            Some((1500, 1000))
        );
    }

    #[test]
    fn a_progressive_photo_whose_coefficients_alone_overflow_is_refused() {
        // 20 MP at 4:4:4 keeps 120 MB of coefficients at any scale.
        let frame = frame(5000, 4000, &SAMPLING_444);

        let eighths = frame.eighths_within((1600, 1280), true, &worker_allowance());

        assert_eq!(eighths, None);
    }

    #[test]
    fn cmyk_ink_becomes_rgb_light() {
        let ink = vec![
            0, 0, 0, 0, // paper
            255, 0, 0, 0, // cyan
            0, 0, 0, 255, // black
            0, 255, 255, 0, // red
        ];

        assert_eq!(
            cmyk_to_rgb(ink),
            [255, 255, 255, 0, 255, 255, 0, 0, 0, 255, 0, 0]
        );
    }

    #[test]
    fn a_lossless_jpeg_is_refused_before_decoding() {
        // A lossless frame header for an 8000×8000 gray image, which would decode at full
        // size into 16-bit samples, about 384 MB with its working buffers.
        let mut file = vec![0xff, 0xd8, 0xff, 0xc3, 0, 11, 16];
        file.extend(8000_u16.to_be_bytes());
        file.extend(8000_u16.to_be_bytes());
        file.extend([1, 1, 0x11, 0]);
        let allowance = Allowance {
            bytes: u64::MAX,
            too_large: Problem::Damaged,
        };

        assert!(matches!(
            decode(&file, 1600, &allowance),
            Err(Problem::Unsupported)
        ));
    }
}
