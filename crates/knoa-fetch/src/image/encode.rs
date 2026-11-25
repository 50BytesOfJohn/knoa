//! Encoding the copies Knoa makes of an image.

use std::borrow::Cow;

use image::{
    DynamicImage, ImageEncoder as _, Rgb, RgbImage,
    codecs::{jpeg::JpegEncoder, png::PngEncoder},
};

/// A JPEG, with any transparency laid on white.
pub fn jpeg(image: &DynamicImage, quality: u8) -> Option<Vec<u8>> {
    // RGB pixels are encoded where they are: a copy would be the largest thing alive.
    let pixels = match image {
        DynamicImage::ImageRgb8(pixels) => Cow::Borrowed(pixels),
        other if other.color().has_alpha() => Cow::Owned(on_white(other)),
        other => Cow::Owned(other.to_rgb8()),
    };
    let mut bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut bytes, quality)
        .encode_image(pixels.as_ref())
        .inspect_err(|err| tracing::debug!("can't encode a JPEG: {err}"))
        .ok()?;
    Some(bytes)
}

pub fn png(image: &DynamicImage) -> Option<Vec<u8>> {
    let pixels = image.to_rgba8();
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(
            &pixels,
            pixels.width(),
            pixels.height(),
            image::ExtendedColorType::Rgba8,
        )
        .inspect_err(|err| tracing::debug!("can't encode a PNG: {err}"))
        .ok()?;
    Some(bytes)
}

/// Whether any pixel is see-through, not only whether the format could hold one.
pub fn has_transparency(image: &DynamicImage) -> bool {
    match image {
        DynamicImage::ImageRgba8(pixels) => pixels.pixels().any(|p| p[3] < u8::MAX),
        DynamicImage::ImageLumaA8(pixels) => pixels.pixels().any(|p| p[1] < u8::MAX),
        other => other.color().has_alpha(),
    }
}

fn on_white(image: &DynamicImage) -> RgbImage {
    let rgba = match image {
        DynamicImage::ImageRgba8(pixels) => Cow::Borrowed(pixels),
        other => Cow::Owned(other.to_rgba8()),
    };
    RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
        let [r, g, b, a] = rgba.get_pixel(x, y).0;
        let blend =
            |c: u8| ((u16::from(c) * u16::from(a) + 255 * (255 - u16::from(a))) / 255) as u8;
        Rgb([blend(r), blend(g), blend(b)])
    })
}
