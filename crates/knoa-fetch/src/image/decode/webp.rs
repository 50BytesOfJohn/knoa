//! WebP files. An animated WebP decodes to its first frame.
//!
//! The container is walked before decoding: image-webp sizes a frame's buffers from the
//! frame's own header before comparing it with the canvas, so a file announcing a 1×1
//! canvas could still ask for gigabytes.

use std::io::Cursor;

use image::{DynamicImage, codecs::webp::WebPDecoder};

use super::{Decoded, Problem, budget::Allowance, still};

pub fn decode(bytes: &[u8], canvas: (u32, u32), allowance: &Allowance) -> Result<Decoded, Problem> {
    let layout = Layout::read(bytes).ok_or(Problem::Damaged)?;
    if layout.canvas != canvas {
        return Err(Problem::Damaged);
    }
    allowance.check(layout.peak_bytes())?;
    let decoder = WebPDecoder::new(Cursor::new(bytes)).map_err(|_| Problem::Damaged)?;
    still(
        DynamicImage::from_decoder(decoder),
        Some(layout.animated),
        allowance,
    )
}

/// What a WebP file holds, read from its chunk headers. Only a file whose every frame fits
/// its canvas is read.
#[derive(Debug, PartialEq, Eq)]
struct Layout {
    canvas: (u32, u32),
    /// Every frame: the one picture of a still, each frame of an animation.
    frames: Vec<Frame>,
    /// From the extended header, or a lossless bitstream's own flag.
    alpha: bool,
    animated: bool,
}

/// One bitstream as its own header sizes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Frame {
    width: u32,
    height: u32,
    lossless: bool,
}

impl Layout {
    fn read(bytes: &[u8]) -> Option<Layout> {
        if bytes.get(..4)? != b"RIFF" || bytes.get(8..12)? != b"WEBP" {
            return None;
        }
        let riff_end = usize::try_from(le(bytes.get(4..8)?))
            .ok()?
            .saturating_add(8);
        let body = bytes.get(12..riff_end.min(bytes.len()))?;
        let mut chunks = Chunks(body).peekable();
        let (first, payload) = chunks.peek().copied()?;
        if first != *b"VP8X" {
            let frame = bitstream(first, payload)?;
            let alpha = frame.lossless && payload.get(4)? & 0x10 != 0;
            return Some(Layout {
                canvas: (frame.width, frame.height),
                frames: vec![frame],
                alpha,
                animated: false,
            });
        }
        let flags = *payload.first()?;
        let canvas = (le(payload.get(4..7)?) + 1, le(payload.get(7..10)?) + 1);
        let animated = flags & 0x02 != 0;
        let frames: Vec<Frame> = chunks
            .skip(1)
            .filter_map(|(id, payload)| match (&id, animated) {
                (b"ANMF", true) => Some(animation_frame(payload, canvas)),
                (b"VP8 " | b"VP8L", false) => Some(bitstream(id, payload)),
                _ => None,
            })
            .collect::<Option<_>>()?;
        if frames.is_empty() || (!animated && frames.iter().any(|f| (f.width, f.height) != canvas))
        {
            return None;
        }
        Some(Layout {
            canvas,
            frames,
            alpha: flags & 0x10 != 0,
            animated,
        })
    }
}

impl Layout {
    /// What decoding the first frame holds at its peak: the output, and the frame's own
    /// buffers. An animation's first frame is also decoded on its own and laid on an RGBA
    /// canvas.
    fn peak_bytes(&self) -> u64 {
        let canvas = pixels(self.canvas.0, self.canvas.1);
        let output = canvas * if self.alpha || self.animated { 4 } else { 3 };
        let frame = self.frames.first().map_or(0, |f| f.peak_bytes(self.alpha));
        if self.animated {
            let first = self.frames.first().map_or(0, |f| pixels(f.width, f.height));
            output + canvas * 4 + first * 4 + frame
        } else {
            output + frame
        }
    }
}

impl Frame {
    /// A lossless frame decodes through an RGBA buffer of its size. A lossy one decodes
    /// into planes of whole 16 pixel macroblocks, 1.5 bytes a pixel; its alpha plane goes
    /// through an RGBA pass of its own.
    fn peak_bytes(self, alpha: bool) -> u64 {
        let size = pixels(self.width, self.height);
        if self.lossless {
            return size * 4;
        }
        let macroblocks = pixels(
            self.width.next_multiple_of(16),
            self.height.next_multiple_of(16),
        );
        macroblocks * 3 / 2 + if alpha { size * 5 } else { 0 }
    }
}

fn pixels(width: u32, height: u32) -> u64 {
    u64::from(width) * u64::from(height)
}

/// An animation frame: placed on the canvas, and holding a bitstream of its own size.
fn animation_frame(payload: &[u8], canvas: (u32, u32)) -> Option<Frame> {
    let x = le(payload.get(0..3)?) * 2;
    let y = le(payload.get(3..6)?) * 2;
    let width = le(payload.get(6..9)?) + 1;
    let height = le(payload.get(9..12)?) + 1;
    if x + width > canvas.0 || y + height > canvas.1 {
        return None;
    }
    let (id, data) = Chunks(payload.get(16..)?).find(|(id, _)| matches!(id, b"VP8 " | b"VP8L"))?;
    bitstream(id, data).filter(|frame| (frame.width, frame.height) == (width, height))
}

/// The size a VP8 or VP8L bitstream's own header gives.
fn bitstream(id: [u8; 4], payload: &[u8]) -> Option<Frame> {
    match &id {
        b"VP8 " => {
            if payload.get(3..6)? != [0x9d, 0x01, 0x2a] {
                return None;
            }
            Some(Frame {
                width: le(payload.get(6..8)?) & 0x3fff,
                height: le(payload.get(8..10)?) & 0x3fff,
                lossless: false,
            })
        }
        b"VP8L" => {
            if *payload.first()? != 0x2f {
                return None;
            }
            let bits = le(payload.get(1..5)?);
            Some(Frame {
                width: (bits & 0x3fff) + 1,
                height: ((bits >> 14) & 0x3fff) + 1,
                lossless: true,
            })
        }
        _ => None,
    }
}

/// A RIFF chunk list: each chunk's id and payload, until the data runs out.
struct Chunks<'a>(&'a [u8]);

impl<'a> Iterator for Chunks<'a> {
    type Item = ([u8; 4], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let id: [u8; 4] = self.0.get(..4)?.try_into().ok()?;
        let size = usize::try_from(le(self.0.get(4..8)?)).ok()?;
        let payload = self.0.get(8..8usize.checked_add(size)?)?;
        // Chunks are padded to an even length.
        let next = (8 + size).next_multiple_of(2).min(self.0.len());
        self.0 = &self.0[next..];
        Some((id, payload))
    }
}

/// A little-endian number of up to four bytes.
fn le(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .rev()
        .fold(0, |n, byte| (n << 8) | u32::from(*byte))
}

#[cfg(test)]
mod tests {
    use image::{ImageEncoder as _, RgbaImage, codecs::webp::WebPEncoder};

    use super::*;

    fn unlimited() -> Allowance {
        Allowance {
            bytes: u64::MAX,
            too_large: Problem::Unsupported,
        }
    }

    fn chunk(id: [u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut chunk = id.to_vec();
        chunk.extend((payload.len() as u32).to_le_bytes());
        chunk.extend(payload);
        if payload.len() % 2 == 1 {
            chunk.push(0);
        }
        chunk
    }

    fn riff(chunks: &[Vec<u8>]) -> Vec<u8> {
        let body: Vec<u8> = chunks.concat();
        let mut file = b"RIFF".to_vec();
        file.extend((body.len() as u32 + 4).to_le_bytes());
        file.extend(b"WEBP");
        file.extend(body);
        file
    }

    #[test]
    fn a_frame_larger_than_its_canvas_is_refused_before_decoding() {
        // An extended header claiming a 1×1 canvas, around a VP8 frame header claiming
        // 16383×16383: image-webp would allocate its planes, about 384 MiB, first.
        let vp8x = [0; 10];
        let mut vp8 = vec![0x10, 0x02, 0x00, 0x9d, 0x01, 0x2a];
        vp8.extend(0x3fff_u16.to_le_bytes());
        vp8.extend(0x3fff_u16.to_le_bytes());
        vp8.extend([0; 16]);
        let file = riff(&[chunk(*b"VP8X", &vp8x), chunk(*b"VP8 ", &vp8)]);

        assert_eq!(Layout::read(&file), None);
        assert!(matches!(
            decode(&file, (1, 1), &unlimited()),
            Err(Problem::Damaged)
        ));
    }

    #[test]
    fn a_real_file_is_read_and_decoded() {
        let pixels = RgbaImage::from_fn(40, 30, |x, y| image::Rgba([x as u8, y as u8, 90, 200]));
        let mut file = Vec::new();
        WebPEncoder::new_lossless(&mut file)
            .write_image(&pixels, 40, 30, image::ExtendedColorType::Rgba8)
            .expect("encodable");

        let layout = Layout::read(&file).expect("a valid layout");

        assert_eq!(layout.canvas, (40, 30));
        assert!(layout.alpha && layout.frames[0].lossless && !layout.animated);
        let decoded = decode(&file, (40, 30), &unlimited()).expect("decodable");
        assert_eq!((decoded.image.width(), decoded.image.height()), (40, 30));
    }
}
