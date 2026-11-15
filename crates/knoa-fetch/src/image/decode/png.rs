//! PNGs. An animated PNG decodes to its first frame.

use std::io::Cursor;

use image::{DynamicImage, ImageDecoder as _, Limits, codecs::png::PngDecoder};

use super::{Decoded, Problem, budget::Allowance, refusal, still};

/// What the decoder may allocate besides the pixels: text and color-profile chunks, which
/// a small file can inflate to hundreds of megabytes, and a few rows. Applied from the
/// first chunk on.
const CHUNK_BYTES: u64 = 4 * 1024 * 1024;

pub fn decode(bytes: &[u8], allowance: &Allowance) -> Result<Decoded, Problem> {
    let mut limits = Limits::default();
    limits.max_alloc = Some(CHUNK_BYTES);
    let decoder = PngDecoder::with_limits(Cursor::new(bytes), limits)
        .map_err(|err| refusal(&err, allowance))?;
    // Only worth a mention when it is: few PNGs are animated.
    let animated = decoder.is_apng().ok().filter(|apng| *apng);
    // `from_decoder` allocates the pixels at their own bit depth without consulting any
    // limit, so they are checked here.
    allowance.check(decoder.total_bytes().saturating_add(CHUNK_BYTES))?;
    still(DynamicImage::from_decoder(decoder), animated, allowance)
}

#[cfg(test)]
mod tests {
    use image::{ExtendedColorType, ImageEncoder as _, codecs::png::PngEncoder};

    use super::*;

    fn allowance(bytes: u64) -> Allowance {
        Allowance {
            bytes,
            too_large: Problem::TooManyPixels {
                width: 0,
                height: 0,
            },
        }
    }

    fn png(width: u32, height: u32, color: ExtendedColorType) -> Vec<u8> {
        let size = width as usize * height as usize * usize::from(color.bits_per_pixel() / 8);
        let mut file = Vec::new();
        PngEncoder::new(&mut file)
            .write_image(&vec![90; size], width, height, color)
            .expect("encodable");
        file
    }

    /// `file` with a chunk inserted after its header.
    fn with_chunk(file: &[u8], kind: [u8; 4], data: &[u8]) -> Vec<u8> {
        let header_end = 8 + 25;
        let mut chunk = (data.len() as u32).to_be_bytes().to_vec();
        chunk.extend(kind);
        chunk.extend(data);
        chunk.extend(crc32(&chunk[4..]).to_be_bytes());
        [&file[..header_end], &chunk, &file[header_end..]].concat()
    }

    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = !0_u32;
        for byte in bytes {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    (crc >> 1) ^ 0xedb8_8320
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }

    #[test]
    fn sixteen_bit_pixels_count_at_their_own_size() {
        // 400×400 is 640 KB of RGBA but 1.28 MB of 16-bit RGBA.
        let budget = allowance(CHUNK_BYTES + 1024 * 1024);

        let rgba8 = decode(&png(400, 400, ExtendedColorType::Rgba8), &budget);
        let rgba16 = decode(&png(400, 400, ExtendedColorType::Rgba16), &budget);

        assert!(rgba8.is_ok());
        assert!(matches!(rgba16, Err(Problem::TooManyPixels { .. })));
    }

    #[test]
    fn a_chunk_past_the_chunk_limit_is_refused() {
        let mut text = b"Comment\0".to_vec();
        text.resize(CHUNK_BYTES as usize + 1, b'x');
        let file = with_chunk(&png(4, 4, ExtendedColorType::Rgb8), *b"tEXt", &text);

        let decoded = decode(&file, &allowance(u64::MAX));

        assert!(matches!(decoded, Err(Problem::TooManyPixels { .. })));
    }
}
