//! Shrinking images to a longest side, never enlarging them.

use image::{DynamicImage, imageops::FilterType};

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

/// `image` shrunk to fit `max_side`, or `None` when it already fits.
pub fn shrunk(image: &DynamicImage, max_side: u32) -> Option<DynamicImage> {
    let (width, height) = fit(image.width(), image.height(), max_side);
    if (width, height) == (image.width(), image.height()) {
        return None;
    }
    // Catmull-Rom is sharp enough for a downscale and much cheaper than Lanczos, which
    // matters in the Worker.
    Some(image.resize_exact(width, height, FilterType::CatmullRom))
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
    fn fit_never_enlarges() {
        assert_eq!(fit(267, 448, 1024), (267, 448));
        assert_eq!(fit(1024, 1024, 1024), (1024, 1024));
    }
}
