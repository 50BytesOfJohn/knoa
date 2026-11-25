//! PNGs, shrunk row by row as they are read, so a large one never exists at full size. An
//! animated PNG decodes to its first frame.

use std::io::Cursor;

use image::{ColorType, DynamicImage, metadata::Orientation};
use png::{BitDepth, Decoder, Limits, Transformations};

use super::{Decoded, Problem, budget::Allowance};
use crate::image::scale::{self, Resampler};

/// What the decoder may allocate besides the pixels: text and color-profile chunks, which
/// a small file can inflate to hundreds of megabytes, and a few rows. Applied from the
/// first chunk on.
const CHUNK_BYTES: u64 = 4 * 1024 * 1024;

pub fn decode(bytes: &[u8], target_side: u32, allowance: &Allowance) -> Result<Decoded, Problem> {
    let mut decoder = Decoder::new_with_limits(
        Cursor::new(bytes),
        Limits {
            bytes: CHUNK_BYTES as usize,
        },
    );
    // Eight bits a channel, palettes and low depths expanded: what the copies need.
    decoder.set_transformations(Transformations::normalize_to_color8());
    let mut reader = decoder
        .read_info()
        .map_err(|err| refusal(&err, allowance))?;
    // Only worth a mention when it is: few PNGs are animated.
    let animated = reader.info().animation_control.is_some().then_some(true);
    let color = match reader.output_color_type() {
        (png::ColorType::Grayscale, BitDepth::Eight) => ColorType::L8,
        (png::ColorType::GrayscaleAlpha, BitDepth::Eight) => ColorType::La8,
        (png::ColorType::Rgb, BitDepth::Eight) => ColorType::Rgb8,
        (png::ColorType::Rgba, BitDepth::Eight) => ColorType::Rgba8,
        _ => return Err(Problem::Unsupported),
    };
    let source = (reader.info().width, reader.info().height);
    let target = scale::fit(source.0, source.1, target_side);
    // Adam7 hands out its rows in seven passes over the picture, so it is read whole.
    let rows = CHUNK_BYTES + row_bytes(reader.info(), color);
    let image = if target == source || reader.info().interlaced {
        let whole = pixels(source) * u64::from(color.bytes_per_pixel());
        allowance.check(rows + whole, whole)?;
        whole_frame(&mut reader, source, color)
    } else {
        let resampler = Resampler::bytes(source, target, color.channel_count());
        allowance.check(rows + resampler, resampler)?;
        shrunk_frame(&mut reader, source, target, color)
    }
    .map_err(|err| refusal(&err, allowance))?;
    let image = image.ok_or(Problem::Damaged)?;
    Ok(Decoded::new(image, Orientation::NoTransforms, animated))
}

type Reader<'a> = png::Reader<Cursor<&'a [u8]>>;

fn whole_frame(
    reader: &mut Reader<'_>,
    (width, height): (u32, u32),
    color: ColorType,
) -> Result<Option<DynamicImage>, png::DecodingError> {
    let mut pixels = vec![0; reader.output_buffer_size().unwrap_or(0)];
    reader.next_frame(&mut pixels)?;
    Ok(scale::from_raw(width, height, color, pixels))
}

fn shrunk_frame(
    reader: &mut Reader<'_>,
    source: (u32, u32),
    target: (u32, u32),
    color: ColorType,
) -> Result<Option<DynamicImage>, png::DecodingError> {
    let mut resampler = Resampler::new(source, target, color);
    while let Some(row) = reader.next_row()? {
        resampler.push_row(row.data());
    }
    Ok(resampler.finish())
}

/// What the reader holds to hand out rows, which no `Limits` bound: its buffer of raw rows,
/// which it trims only once about four rows are spent (never past the whole image) and
/// which grows by doubling, so up to twice its fill and, while it moves, the old block too;
/// a copy of the previous row; and the row in eight bits a channel.
fn row_bytes(info: &png::Info<'_>, color: ColorType) -> u64 {
    let width = u64::from(info.width);
    let bits = width * info.color_type.samples() as u64 * u64::from(info.bit_depth as u8);
    // A filter byte leads every row.
    let raw_row = 1 + bits.div_ceil(8);
    let trimmed_after = (4 * raw_row).next_multiple_of(64).max(128 * 1024);
    let filled = (trimmed_after + 3 * raw_row).min(u64::from(info.height) * raw_row) + 8 * 1024;
    3 * filled + raw_row + width * u64::from(color.bytes_per_pixel())
}

fn pixels((width, height): (u32, u32)) -> u64 {
    u64::from(width) * u64::from(height)
}

/// A decoder that ran into its limit was refused for size; any other failure is damage.
fn refusal(err: &png::DecodingError, allowance: &Allowance) -> Problem {
    match err {
        png::DecodingError::LimitsExceeded => allowance.too_large.clone(),
        _ => Problem::Damaged,
    }
}

#[cfg(test)]
mod tests {
    use image::{ExtendedColorType, ImageEncoder as _, codecs::png::PngEncoder};

    use super::{super::budget::WORKER, *};

    fn unlimited() -> Allowance {
        Allowance::unlimited(Problem::TooManyPixels {
            width: 0,
            height: 0,
        })
    }

    fn png(width: u32, height: u32, color: ExtendedColorType) -> Vec<u8> {
        let size = width as usize * height as usize * usize::from(color.bits_per_pixel() / 8);
        let mut file = Vec::new();
        PngEncoder::new(&mut file)
            .write_image(&vec![90; size], width, height, color)
            .expect("encodable");
        file
    }

    /// A black 16-bit RGBA PNG, written a row at a time: a large one is a small file.
    fn blank_rgba16(width: u32, height: u32) -> Vec<u8> {
        let mut file = Vec::new();
        let mut encoder = png::Encoder::new(&mut file, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(BitDepth::Sixteen);
        encoder.set_compression(png::Compression::Fastest);
        let mut writer = encoder.write_header().expect("a header");
        let mut rows = writer.stream_writer().expect("a writer");
        let row = vec![0; width as usize * 8];
        for _ in 0..height {
            std::io::Write::write_all(&mut rows, &row).expect("writable");
        }
        rows.finish().expect("finished");
        drop(writer);
        file
    }

    /// `file` with a chunk inserted after its header.
    fn with_chunk(file: &[u8], kind: [u8; 4], data: &[u8]) -> Vec<u8> {
        let header_end = 8 + 25;
        [&file[..header_end], &chunk(kind, data), &file[header_end..]].concat()
    }

    fn chunk(kind: [u8; 4], data: &[u8]) -> Vec<u8> {
        let mut chunk = (data.len() as u32).to_be_bytes().to_vec();
        chunk.extend(kind);
        chunk.extend(data);
        chunk.extend(crc32(&chunk[4..]).to_be_bytes());
        chunk
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
    fn a_large_png_is_shrunk_as_it_is_read_into_eight_bits() {
        let file = png(400, 200, ExtendedColorType::Rgba16);

        let decoded = decode(&file, 100, &unlimited()).expect("decodable");

        let image = decoded.upright(100);
        assert_eq!((image.width(), image.height()), (100, 50));
        assert_eq!(image.color(), ColorType::Rgba8);
        assert_eq!(image.to_rgba8().get_pixel(50, 25).0, [90; 4]);
    }

    #[test]
    fn a_wide_png_whose_rows_overflow_the_worker_is_refused_before_decoding() {
        // Rows of 6.4 MB: the reader's buffer of them grows to 64 MiB, though the file is
        // small and the shrunk copy tiny.
        let (width, height) = (800_000, 32);
        let file = blank_rgba16(width, height);
        let too_large = Problem::TooManyPixels { width, height };
        let worker = Allowance::new(WORKER, file.len() as u64, 0, too_large.clone());

        let decoded = decode(&file, 1600, &worker);

        assert_eq!(decoded.err(), Some(too_large));
    }

    #[test]
    fn a_sliver_70_million_pixels_long_fits_the_desktop_budget() {
        // Only a header and a scrap of image data: it passes the size checks, then fails
        // as damaged. Weights for every output row would have taken 1.1 GB.
        let mut file = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = 1_u32.to_be_bytes().to_vec();
        header.extend(70_000_000_u32.to_be_bytes());
        header.extend([8, 0, 0, 0, 0]);
        file.extend(chunk(*b"IHDR", &header));
        file.extend(chunk(*b"IDAT", &[0x78, 0x9c, 0x63]));

        let decoded = crate::image::decode::decode(
            &file,
            crate::image::facts::Codec::Png,
            (1, 70_000_000),
            1600,
            crate::image::decode::DESKTOP,
        );

        assert_eq!(decoded.err(), Some(Problem::Damaged));
    }

    #[test]
    fn a_chunk_past_the_chunk_limit_is_refused() {
        let mut text = b"Comment\0".to_vec();
        text.resize(CHUNK_BYTES as usize + 1, b'x');
        let file = with_chunk(&png(4, 4, ExtendedColorType::Rgb8), *b"tEXt", &text);

        let decoded = decode(&file, 1600, &unlimited());

        assert!(matches!(decoded, Err(Problem::TooManyPixels { .. })));
    }
}
