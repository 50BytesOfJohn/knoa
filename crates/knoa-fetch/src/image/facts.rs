//! What an image file's header says about it, read without decoding.

use serde::Serialize;

/// The formats Knoa decodes. Others are described from their header only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    Jpeg,
    Png,
    Gif,
    Webp,
}

impl Codec {
    pub fn extension(self) -> &'static str {
        match self {
            Codec::Jpeg => "jpg",
            Codec::Png => "png",
            Codec::Gif => "gif",
            Codec::Webp => "webp",
        }
    }
}

/// An image file's details, as the AI sees them in the evidence.
#[derive(Debug, Clone, Serialize)]
pub struct Facts {
    /// `JPEG`, `PNG`, `AVIF`, ...
    pub format: String,
    /// As shown: after the photo's orientation is applied, when that is known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// The file's size, when it was downloaded in full.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<usize>,
    /// Known once the file is decoded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animated: Option<bool>,
    #[serde(skip)]
    pub codec: Option<Codec>,
}

impl Facts {
    /// Reads `body`'s header. `complete` is false when the body is only the file's start,
    /// or nothing at all.
    pub fn read(body: &[u8], content_type: &str, complete: bool) -> Facts {
        let codec = match imagesize::image_type(body) {
            Ok(imagesize::ImageType::Jpeg) => Some(Codec::Jpeg),
            Ok(imagesize::ImageType::Png) => Some(Codec::Png),
            Ok(imagesize::ImageType::Gif) => Some(Codec::Gif),
            Ok(imagesize::ImageType::Webp) => Some(Codec::Webp),
            _ => None,
        };
        let size = imagesize::blob_size(body).ok();
        Facts {
            format: codec.map_or_else(|| format_of(content_type), |c| name(c).to_owned()),
            width: size.map(|s| s.width as u32),
            height: size.map(|s| s.height as u32),
            bytes: complete.then_some(body.len()),
            animated: None,
            codec,
        }
    }

    /// `3840×2160`, the image kind's resolution field.
    pub fn resolution(&self) -> Option<String> {
        Some(format!("{}×{}", self.width?, self.height?))
    }
}

fn name(codec: Codec) -> &'static str {
    match codec {
        Codec::Jpeg => "JPEG",
        Codec::Png => "PNG",
        Codec::Gif => "GIF",
        Codec::Webp => "WebP",
    }
}

/// `image/svg+xml` → `SVG`, `image/x-icon` → `ICON`.
fn format_of(content_type: &str) -> String {
    let subtype = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_start_matches("image/");
    let subtype = subtype.trim_start_matches("x-").trim_end_matches("+xml");
    subtype.to_ascii_uppercase()
}
