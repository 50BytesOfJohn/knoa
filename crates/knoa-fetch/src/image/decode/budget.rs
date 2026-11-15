//! Pixel budgets, checked against the header before any decoding. The cloud organizer's
//! 128 MB Worker sets them; the Mac has memory to spare and decodes far larger files, but
//! stays bounded so a hostile header can't ask for gigabytes.

#[cfg(target_arch = "wasm32")]
mod target {
    /// Baseline JPEGs decode a row at a time and are scaled down in the DCT, so even a
    /// 64 MP photo decodes into a few megabytes.
    pub const JPEG_PIXELS: u64 = 64_000_000;
    /// Progressive JPEGs hold every coefficient until the last scan, about 3 bytes a pixel
    /// at full size whatever the scale.
    pub const PROGRESSIVE_JPEG_PIXELS: u64 = 16_000_000;
    /// PNG, WebP and GIF decode at full size, up to 4 bytes a pixel: 64 MB at this budget.
    pub const OTHER_PIXELS: u64 = 16_000_000;
    /// `image`'s own cap on a decoder's allocations: the backstop for 16-bit PNGs, which
    /// take twice the bytes the pixel budget assumes.
    pub const MAX_ALLOC: u64 = 80 * 1024 * 1024;
}

#[cfg(not(target_arch = "wasm32"))]
mod target {
    /// 100 MP covers 8K and 10K wallpapers and most camera files in every format.
    pub const JPEG_PIXELS: u64 = 100_000_000;
    pub const PROGRESSIVE_JPEG_PIXELS: u64 = 100_000_000;
    pub const OTHER_PIXELS: u64 = 100_000_000;
    /// A 16-bit RGBA PNG at the pixel budget is 800 MB, decoded once and dropped.
    pub const MAX_ALLOC: u64 = 1024 * 1024 * 1024;
}

pub use target::{JPEG_PIXELS, MAX_ALLOC, OTHER_PIXELS, PROGRESSIVE_JPEG_PIXELS};
