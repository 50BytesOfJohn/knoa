//! GIFs: the first frame, laid on a transparent canvas.

use std::io::Cursor;

use image::{DynamicImage, codecs::gif::GifDecoder};

use super::{Decoded, Problem, budget::Allowance, still};

pub fn decode(bytes: &[u8], canvas: (u32, u32), allowance: &Allowance) -> Result<Decoded, Problem> {
    let layout = Layout::read(bytes).ok_or(Problem::Damaged)?;
    if layout.canvas != canvas {
        return Err(Problem::Damaged);
    }
    allowance.check(layout.peak_bytes(), layout.output_bytes())?;
    let decoder = GifDecoder::new(Cursor::new(bytes)).map_err(|_| Problem::Damaged)?;
    still(
        DynamicImage::from_decoder(decoder),
        Some(layout.animated),
        allowance,
    )
}

/// What a GIF's blocks say before any pixel is decoded.
#[derive(Debug, PartialEq, Eq)]
struct Layout {
    canvas: (u32, u32),
    /// The first frame's size, which may differ from the canvas.
    first: (u32, u32),
    animated: bool,
}

impl Layout {
    /// Walks the blocks as far as a second frame, which makes the GIF an animation.
    fn read(bytes: &[u8]) -> Option<Layout> {
        if !matches!(bytes.get(..6)?, b"GIF87a" | b"GIF89a") {
            return None;
        }
        let canvas = (le(bytes.get(6..8)?), le(bytes.get(8..10)?));
        let mut at = 13 + color_table(*bytes.get(10)?);
        let mut first = None;
        loop {
            match *bytes.get(at)? {
                // An extension: its label, then data sub-blocks.
                0x21 => at = skip_sub_blocks(bytes, at + 2)?,
                0x2c => {
                    let size = (
                        le(bytes.get(at + 5..at + 7)?),
                        le(bytes.get(at + 7..at + 9)?),
                    );
                    if first.is_some() {
                        return Some(Layout {
                            canvas,
                            first: first?,
                            animated: true,
                        });
                    }
                    first = Some(size);
                    // The descriptor, its color table and the LZW code size, then the data.
                    let table = color_table(*bytes.get(at + 9)?);
                    at = skip_sub_blocks(bytes, at + 10 + table + 1)?;
                }
                // The trailer, or anything else ends the frames, as decoders treat it.
                _ => break,
            }
        }
        Some(Layout {
            canvas,
            first: first?,
            animated: false,
        })
    }

    /// The RGBA output, and the first frame's own RGBA buffer and color indices when it
    /// doesn't cover the canvas's full width.
    fn peak_bytes(&self) -> u64 {
        self.output_bytes() + pixels(self.first) * 5
    }

    /// The canvas, in RGBA.
    fn output_bytes(&self) -> u64 {
        pixels(self.canvas) * 4
    }
}

fn pixels((width, height): (u32, u32)) -> u64 {
    u64::from(width) * u64::from(height)
}

/// The bytes of a color table, from the flags byte announcing it.
fn color_table(flags: u8) -> usize {
    if flags & 0x80 == 0 {
        0
    } else {
        3 << ((flags & 0x07) + 1)
    }
}

/// The position after the sub-blocks starting at `at`, which end with an empty one.
fn skip_sub_blocks(bytes: &[u8], mut at: usize) -> Option<usize> {
    loop {
        let length = usize::from(*bytes.get(at)?);
        at += 1 + length;
        if length == 0 {
            return Some(at);
        }
    }
}

fn le(bytes: &[u8]) -> u32 {
    u32::from(u16::from_le_bytes([bytes[0], bytes[1]]))
}
