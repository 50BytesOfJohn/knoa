//! What the main window shows behind its floating panels: the theme's canvas, the
//! desktop through a material, or a picture.

use std::path::{Path, PathBuf};

use knoa_core::unknown_as_default;
use serde::{Deserialize, Deserializer, Serialize};

/// An unknown style or effect, say from a newer version, falls back to its default
/// alone rather than invalidating the whole settings file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Background {
    #[serde(deserialize_with = "unknown_as_default")]
    pub style: BackgroundStyle,
    #[serde(deserialize_with = "unknown_as_default")]
    pub effect: Effect,
    /// How much of the canvas colour covers the effect, from 0 to 1.
    #[serde(deserialize_with = "clamped")]
    pub opacity: f32,
    pub image: Option<BackgroundImage>,
    /// How much of the picture covers the canvas colour, from 0 to 1.
    #[serde(deserialize_with = "clamped")]
    pub image_opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackgroundStyle {
    /// The theme's canvas colour.
    #[default]
    Solid,
    /// The desktop through [`Effect`], under the canvas colour at [`Background::opacity`].
    Translucent,
    /// [`Background::image`] over the canvas colour.
    Image,
}

/// The material a translucent window shows the desktop through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effect {
    /// macOS's own vibrancy, tinted by the desktop.
    #[default]
    Frosted,
    /// A neutral blur.
    Blur,
    /// No blur at all.
    Clear,
}

/// The picture behind the panels: Knoa's own copy, and the name of the file it was
/// made from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundImage {
    pub path: PathBuf,
    pub name: String,
}

impl Default for Background {
    fn default() -> Background {
        Background {
            style: BackgroundStyle::default(),
            effect: Effect::default(),
            opacity: 0.6,
            image: None,
            image_opacity: 1.,
        }
    }
}

impl Background {
    /// The picture to show, while the style shows one.
    pub fn shown_image(&self) -> Option<&Path> {
        match self.style {
            BackgroundStyle::Image => self.image.as_ref().map(|image| image.path.as_path()),
            BackgroundStyle::Solid | BackgroundStyle::Translucent => None,
        }
    }
}

fn clamped<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    Ok(f32::deserialize(deserializer)?.clamp(0., 1.))
}
