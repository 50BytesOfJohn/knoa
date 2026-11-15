//! Shrinking images to a longest side, never enlarging them.

use image::{
    DynamicImage,
    imageops::{self, FilterType},
};

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

/// How far past the target a box filter takes a large image first. The final filter
/// works in 16-byte float RGBA over the source's width and the target's height: from a box
/// reduction to 1.25 times a 1600 pixel square that is 51 MB, whatever the original's size,
/// where a 4000 pixel wide original took 102 MB.
const BOXED_TO: f64 = 1.25;

/// `image` shrunk to fit `max_side`, or as it is when it already fits. It is dropped as
/// soon as a box reduction replaces it.
pub fn shrink(image: DynamicImage, max_side: u32) -> DynamicImage {
    let (width, height) = fit(image.width(), image.height(), max_side);
    if (width, height) == (image.width(), image.height()) {
        return image;
    }
    let boxed_side = (f64::from(max_side) * BOXED_TO) as u32;
    let image = if image.width().max(image.height()) > boxed_side {
        let (boxed_width, boxed_height) = fit(image.width(), image.height(), boxed_side);
        let boxed = box_filter(&image, boxed_width, boxed_height);
        drop(image);
        boxed
    } else {
        eight_bit(image)
    };
    // Catmull-Rom is sharp enough for a downscale and much cheaper than Lanczos, which
    // matters in the Worker.
    image.resize_exact(width, height, FilterType::CatmullRom)
}

/// Each pixel the average of the area it covers, in 8 bits a channel whatever the source's
/// depth, with no buffer beyond the result.
fn box_filter(image: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    match image {
        DynamicImage::ImageLuma8(_)
        | DynamicImage::ImageLumaA8(_)
        | DynamicImage::ImageRgb8(_)
        | DynamicImage::ImageRgba8(_) => image.thumbnail_exact(width, height),
        other => imageops::thumbnail(other, width, height).into(),
    }
}

/// A 16-bit or float image would keep its depth through the filter, doubling its result.
fn eight_bit(image: DynamicImage) -> DynamicImage {
    match image {
        DynamicImage::ImageLuma8(_)
        | DynamicImage::ImageLumaA8(_)
        | DynamicImage::ImageRgb8(_)
        | DynamicImage::ImageRgba8(_) => image,
        other => other.to_rgba8().into(),
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
    fn fit_never_enlarges() {
        assert_eq!(fit(267, 448, 1024), (267, 448));
        assert_eq!(fit(1024, 1024, 1024), (1024, 1024));
    }
}
