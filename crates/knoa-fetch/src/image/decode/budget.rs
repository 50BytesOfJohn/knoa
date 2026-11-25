//! How much memory one decode may take, checked against the file's headers before any
//! pixels are allocated.
//!
//! The pipeline's peak has two stages, and each stays within the budget's `peak`:
//!
//! - decoding: the body, which the decoder reads in place, and the decoder's peak. A PNG
//!   shrinks its rows as they are read, so its peak includes the shrinking;
//! - shrinking, with the body dropped: the decoded pixels and what the resampler holds,
//!   the shrunk copy and a few rows (at most 10.6 MB for 1600 pixels of RGBA). An EXIF
//!   turn comes after, and copies only the shrunk image.
//!
//! So a decoder may take the peak less the body, and its output the peak less what
//! shrinking (or turning an image that needs no shrinking) holds.
//!
//! Each format estimates its decoder's peak from its own headers:
//!
//! - baseline JPEG: each component's plane at the scaled size and the interleaved pixels;
//!   a photo whose full size doesn't fit decodes at a smaller DCT scale;
//! - progressive JPEG: also every coefficient at full size, two bytes per sample, so 3
//!   bytes a pixel at 4:2:0 and 6 at 4:4:4 at any scale: in the Worker about 7 MP at
//!   4:2:0 and 3.5 MP at 4:4:4;
//! - PNG: what its chunks may take and the resampler, whatever the size; an interlaced
//!   PNG is read whole first, at a byte a sample;
//! - WebP: the output and the frame's planes, 4.5 bytes a pixel for a lossy still (5 MP)
//!   up to about 19 for an animation, which also holds a canvas and a frame (1.2 MP);
//! - GIF: the RGBA output and the first frame, about 9 bytes a pixel (2.6 MP).

#[cfg(any(test, target_arch = "wasm32"))]
const MB: u64 = 1_000_000;

/// What one image may take.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    /// Pixels the header may claim.
    pub pixels: u64,
    /// Bytes the whole pipeline may hold at once: the file and the decoder, or the
    /// decoded pixels and what shrinking or turning them adds.
    pub peak: u64,
}

/// The cloud organizer's Worker has 128 MB for the isolate, shared with JS, and WASM
/// memory only grows: what a job reaches stays for the jobs after it. This keeps an image
/// job's linear memory under 48 MiB: about 3 MiB the organizer holds anyway, the peak, and
/// the shrunk copy again when the allocator can't fit it in a block the decode freed.
#[cfg(any(test, target_arch = "wasm32"))]
pub const WORKER: Budget = Budget {
    pixels: 64_000_000,
    peak: 34 * MB,
};

/// 100 MP covers 8K and 10K wallpapers and most camera files; a decoder may take 1 GiB
/// beside the largest body, enough for any of them decoded whole.
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
    /// What the decoder may hold at its peak, beside the body.
    decoding: u64,
    /// What its output may take, beside what turning it upright holds.
    output: u64,
    pub too_large: super::Problem,
}

impl Allowance {
    /// What `budget` leaves a decoder beside a body of `held` bytes, followed by
    /// `upright` bytes of shrinking or turning.
    pub fn new(budget: Budget, held: u64, upright: u64, too_large: super::Problem) -> Allowance {
        Allowance {
            decoding: budget.peak.saturating_sub(held),
            output: budget.peak.saturating_sub(upright),
            too_large,
        }
    }

    /// Room for any decode, for tests of what isn't about size.
    #[cfg(test)]
    pub fn unlimited(too_large: super::Problem) -> Allowance {
        Allowance {
            decoding: u64::MAX,
            output: u64::MAX,
            too_large,
        }
    }

    /// Whether a decoder holding `peak` bytes at most, `output` of them its result, fits.
    pub fn fits(&self, peak: u64, output: u64) -> bool {
        peak <= self.decoding && output <= self.output
    }

    /// Refuses a decode whose estimated peak or output doesn't fit.
    pub fn check(&self, peak: u64, output: u64) -> Result<(), super::Problem> {
        if self.fits(peak, output) {
            Ok(())
        } else {
            Err(self.too_large.clone())
        }
    }
}
