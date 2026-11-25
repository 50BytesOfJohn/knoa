//! Shrinking images to a longest side, never enlarging them.

mod resampler;

use image::{ColorType, DynamicImage, GrayAlphaImage, GrayImage, RgbImage, RgbaImage};
pub use resampler::Resampler;

/// The size that fits `width`×`height` within `max_side` on its longer side, keeping the
/// aspect ratio. An image that already fits keeps its size.
pub fn fit(width: u32, height: u32, max_side: u32) -> (u32, u32) {
    let longest = width.max(height);
    if longest <= max_side {
        return (width, height);
    }
    let scale = f64::from(max_side) / f64::from(longest);
    let side = |n: u32| ((f64::from(n) * scale).round() as u32).clamp(1, max_side);
    (side(width), side(height))
}

/// `image` shrunk to fit `max_side` in 8 bits a channel, or as it is when it already fits.
pub fn shrink(image: DynamicImage, max_side: u32) -> DynamicImage {
    let source = (image.width(), image.height());
    let target = fit(source.0, source.1, max_side);
    if target == source {
        return image;
    }
    let image = eight_bit(image);
    let color = image.color();
    let mut resampler = Resampler::new(source, target, color);
    let row_len = source.0 as usize * usize::from(color.bytes_per_pixel());
    for row in image.as_bytes().chunks_exact(row_len) {
        resampler.push_row(row);
    }
    resampler
        .finish()
        .expect("every row of the source was pushed")
}

/// `pixels` of an 8-bit `color` as an image; `None` when they don't fill `width`×`height`.
pub fn from_raw(
    width: u32,
    height: u32,
    color: ColorType,
    pixels: Vec<u8>,
) -> Option<DynamicImage> {
    match color {
        ColorType::L8 => GrayImage::from_raw(width, height, pixels).map(Into::into),
        ColorType::La8 => GrayAlphaImage::from_raw(width, height, pixels).map(Into::into),
        ColorType::Rgb8 => RgbImage::from_raw(width, height, pixels).map(Into::into),
        _ => RgbaImage::from_raw(width, height, pixels).map(Into::into),
    }
}

/// The decoders hand out 8-bit pixels; anything deeper is brought down to them first.
fn eight_bit(image: DynamicImage) -> DynamicImage {
    match image.color() {
        ColorType::L8 | ColorType::La8 | ColorType::Rgb8 | ColorType::Rgba8 => image,
        _ => image.to_rgba8().into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_shrinks_the_longer_side_and_keeps_the_aspect_ratio() {
        assert_eq!(fit(3840, 2160, 1024), (1024, 576));
        assert_eq!(fit(2304, 3072, 1600), (1200, 1600));
        assert_eq!(fit(20_000, 10, 1024), (1024, 1), "a sliver keeps a pixel");
    }

    #[test]
    fn a_large_image_shrinks_to_the_exact_size() {
        let wide = DynamicImage::new_rgb16(5000, 2000);

        let shrunk = shrink(wide, 1600);

        assert_eq!((shrunk.width(), shrunk.height()), (1600, 640));
        assert!(matches!(shrunk, DynamicImage::ImageRgba8(_)));
    }

    #[test]
    fn fine_detail_averages_out_instead_of_aliasing() {
        // A one-pixel checkerboard shrunk tenfold: sampling would leave black, white or a
        // moiré; filtering over every source pixel leaves an even gray.
        let checkers =
            GrayImage::from_fn(1000, 1000, |x, y| image::Luma([((x + y) % 2 * 255) as u8]));

        let shrunk = shrink(checkers.into(), 100).to_luma8();

        assert!(shrunk.pixels().all(|p| (120..=135).contains(&p[0])));
    }

    #[test]
    fn fit_never_enlarges() {
        assert_eq!(fit(267, 448, 1024), (267, 448));
        assert_eq!(fit(1024, 1024, 1024), (1024, 1024));
    }
}
