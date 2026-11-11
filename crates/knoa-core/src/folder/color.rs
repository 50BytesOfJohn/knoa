//! Folder and workspace colors: the theme's named inks, or a hue of the user's own.

/// A folder's color. Either way the theme picks the final shade, so folders follow it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FolderColor {
    Ink(Ink),
    Hue(Hue),
}

impl Default for FolderColor {
    fn default() -> FolderColor {
        FolderColor::Ink(Ink::default())
    }
}

/// The named colors every theme provides, in palette order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Ink {
    #[default]
    Ochre,
    Olive,
    Moss,
    Pine,
    Teal,
    Sky,
    Cobalt,
    Violet,
    Plum,
    Rose,
    Cocoa,
    Slate,
}

impl Ink {
    pub const COUNT: usize = 12;
    pub const ALL: [Ink; Ink::COUNT] = [
        Ink::Ochre,
        Ink::Olive,
        Ink::Moss,
        Ink::Pine,
        Ink::Teal,
        Ink::Sky,
        Ink::Cobalt,
        Ink::Violet,
        Ink::Plum,
        Ink::Rose,
        Ink::Cocoa,
        Ink::Slate,
    ];

    /// The ink at `n` in palette order, wrapping around. For rotations, hashes and
    /// stored indexes, so any number names some ink.
    pub fn nth(n: usize) -> Ink {
        Ink::ALL[n % Ink::COUNT]
    }

    /// Pine, Cocoa and Slate: muted on purpose, beside the vivid rest.
    pub fn is_quiet(self) -> bool {
        matches!(self, Ink::Pine | Ink::Cocoa | Ink::Slate)
    }

    /// Its place in the palette, which is also how it is stored.
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn label(self) -> &'static str {
        match self {
            Ink::Ochre => "Ochre",
            Ink::Olive => "Olive",
            Ink::Moss => "Moss",
            Ink::Pine => "Pine",
            Ink::Teal => "Teal",
            Ink::Sky => "Sky",
            Ink::Cobalt => "Cobalt",
            Ink::Violet => "Violet",
            Ink::Plum => "Plum",
            Ink::Rose => "Rose",
            Ink::Cocoa => "Cocoa",
            Ink::Slate => "Slate",
        }
    }
}

/// A hue in whole degrees. Red and orange are left out: they mark the AI, and a folder
/// must never pass for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hue(u16);

impl Hue {
    pub const MIN: u16 = 45;
    /// Past this the theme's shade of pink turns crimson, too close to the AI's red.
    pub const MAX: u16 = 330;

    /// `None` outside [`Hue::MIN`]..=[`Hue::MAX`].
    pub fn new(degrees: u16) -> Option<Hue> {
        (Hue::MIN..=Hue::MAX)
            .contains(&degrees)
            .then_some(Hue(degrees))
    }

    /// The nearest allowed hue, for values from a continuous control like a slider.
    pub fn clamped(degrees: f32) -> Hue {
        Hue(degrees
            .round()
            .clamp(f32::from(Hue::MIN), f32::from(Hue::MAX)) as u16)
    }

    pub fn degrees(self) -> u16 {
        self.0
    }
}
