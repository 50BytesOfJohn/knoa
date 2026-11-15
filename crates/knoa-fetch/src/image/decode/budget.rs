//! How much memory one decode may take, checked against the file's headers before any
//! pixels are allocated.
//!
//! In the cloud organizer's 128 MB Worker the pipeline's peak has three stages, each under
//! about 100 MB:
//!
//! - decoding: the body (up to 24 MiB, read straight from it) and the decoder's peak (at
//!   most `bytes`, 64 MiB): 92 MB;
//! - shrinking, with the body dropped: the decoded pixels (within `bytes`) and their box
//!   reduction to at most 2000 pixels, 16 MB in RGBA: 83 MB;
//! - the final filter, with the decoded pixels dropped: the 16 MB reduction, `image`'s
//!   float pass of 2000×1600 at 16 bytes a pixel (51 MB) and the 10 MB result: 77 MB.
//!
//! Each format estimates its decoder's peak from its own headers:
//!
//! - baseline JPEG: each component's plane at the scaled size, the interleaved pixels and
//!   an EXIF turn's copy, so even a 64 MP photo needs about 25 MB;
//! - progressive JPEG: also every coefficient at full size, two bytes per sample: 3 bytes
//!   a pixel at 4:2:0 and 6 at 4:4:4, about 13 MP and 8 MP;
//! - PNG: the decoded pixels at their bit depth (8 bytes a pixel for 16-bit RGBA) plus
//!   what its chunks may take: 15 MP of RGBA, 7.5 MP at 16 bits;
//! - WebP: the output and the frame's planes, 4.5 bytes a pixel for a lossy still (14 MP)
//!   up to about 19 for an animation, which also holds a canvas and a frame (3.5 MP);
//! - GIF: the RGBA output and the first frame, about 9 bytes a pixel (7 MP).

const MIB: u64 = 1024 * 1024;

/// What one decode may take.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    /// Pixels the header may claim.
    pub pixels: u64,
    /// Bytes the decoder may hold at its peak besides the file itself: its output, its
    /// working buffers and an EXIF turn's copy.
    pub bytes: u64,
}

#[cfg(target_arch = "wasm32")]
pub const BUDGET: Budget = Budget {
    pixels: 64_000_000,
    bytes: 64 * MIB,
};

/// 100 MP covers 8K and 10K wallpapers and most camera files; a 16-bit RGBA PNG that size
/// is 800 MB, decoded once and dropped.
#[cfg(not(target_arch = "wasm32"))]
pub const BUDGET: Budget = Budget {
    pixels: 100_000_000,
    bytes: 1024 * MIB,
};

/// The bytes left for one decode, and how to refuse it.
pub struct Allowance {
    pub bytes: u64,
    pub too_large: super::Problem,
}

impl Allowance {
    /// Refuses a decode whose estimated peak doesn't fit.
    pub fn check(&self, needed: u64) -> Result<(), super::Problem> {
        if needed <= self.bytes {
            Ok(())
        } else {
            Err(self.too_large.clone())
        }
    }
}
