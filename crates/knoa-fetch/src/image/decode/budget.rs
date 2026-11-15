//! How much memory one decode may take, checked against the file's headers before any
//! pixels are allocated.
//!
//! The pipeline's peak has three stages, and each stays within the budget's `peak`:
//!
//! - decoding: the body, which the decoder reads in place, and the decoder's peak;
//! - shrinking, with the body dropped: the decoded pixels (within the decoder's peak) and
//!   their box reduction to at most 2000 pixels, [`REDUCTION_BYTES`] in RGBA;
//! - the final filter, with the decoded pixels dropped: the 16 MB reduction, `image`'s
//!   float pass of 2000×1600 at 16 bytes a pixel (51 MB) and the 10 MB result: 77 MB.
//!
//! So a decoder may take the peak less the body or the reduction, whichever is larger. In
//! the Worker that is 84 MB for a body up to 16 MB and 75 MB beside the largest, 24 MiB.
//!
//! Each format estimates its decoder's peak from its own headers:
//!
//! - baseline JPEG: each component's plane at the scaled size, the interleaved pixels and
//!   an EXIF turn's copy, so even a 64 MP photo needs about 25 MB;
//! - progressive JPEG: also every coefficient at full size, two bytes per sample, so 3
//!   bytes a pixel at 4:2:0 and 6 at 4:4:4 at any scale. In the Worker that decodes 8 MP
//!   at full size; a larger file decodes at a smaller DCT scale, below the display size,
//!   up to about 24 MP at 4:2:0 and 13 MP at 4:4:4 at a quarter of the size;
//! - PNG: the decoded pixels at their bit depth (8 bytes a pixel for 16-bit RGBA) plus
//!   what its chunks may take: in the Worker 20 MP of RGBA, 10 MP at 16 bits;
//! - WebP: the output and the frame's planes, 4.5 bytes a pixel for a lossy still (18 MP)
//!   up to about 19 for an animation, which also holds a canvas and a frame (4.4 MP);
//! - GIF: the RGBA output and the first frame, about 9 bytes a pixel (9 MP).

const MB: u64 = 1_000_000;

/// A box reduction's largest result, 2000×2000 RGBA, held beside the decoded pixels.
const REDUCTION_BYTES: u64 = 16 * MB;

/// What one image may take.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    /// Pixels the header may claim.
    pub pixels: u64,
    /// Bytes the whole pipeline may hold at once: the file, the decoder's output, its
    /// working buffers and an EXIF turn's copy, and what shrinking adds.
    pub peak: u64,
}

/// The cloud organizer's Worker has 128 MB for the isolate. Its compiled module, the JS
/// runtime, the request's other state and the allocator's slack fit in the 28 MB left.
#[cfg(any(test, target_arch = "wasm32"))]
pub const WORKER: Budget = Budget {
    pixels: 64_000_000,
    peak: 100 * MB,
};

/// 100 MP covers 8K and 10K wallpapers and most camera files; a decoder may take 1 GiB
/// beside the largest body, enough for a 16-bit RGBA PNG of 100 MP, decoded once and
/// dropped.
#[cfg(any(test, not(target_arch = "wasm32")))]
pub const DESKTOP: Budget = Budget {
    pixels: 100_000_000,
    peak: (1 << 30) + crate::http::RAW_IMAGE_LIMIT as u64,
};

#[cfg(target_arch = "wasm32")]
pub const BUDGET: Budget = WORKER;
#[cfg(not(target_arch = "wasm32"))]
pub const BUDGET: Budget = DESKTOP;

/// The bytes left for one decode, and how to refuse it.
pub struct Allowance {
    pub bytes: u64,
    pub too_large: super::Problem,
}

impl Allowance {
    /// What `budget` leaves a decoder beside a body of `held` bytes.
    pub fn new(budget: Budget, held: u64, too_large: super::Problem) -> Allowance {
        Allowance {
            bytes: budget.peak.saturating_sub(held.max(REDUCTION_BYTES)),
            too_large,
        }
    }

    pub fn fits(&self, needed: u64) -> bool {
        needed <= self.bytes
    }

    /// Refuses a decode whose estimated peak doesn't fit.
    pub fn check(&self, needed: u64) -> Result<(), super::Problem> {
        if self.fits(needed) {
            Ok(())
        } else {
            Err(self.too_large.clone())
        }
    }
}
