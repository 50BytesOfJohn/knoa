//! What a folder shows instead of the folder shape: one of Knoa's glyphs or an emoji.

use super::emoji::Emoji;

macro_rules! folder_icons {
    ($($variant:ident => $name:literal, $label:literal;)+) => {
        /// A glyph from Knoa's own icon set.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum FolderGlyph {
            $($variant,)+
        }

        impl FolderGlyph {
            /// Every glyph, in the order the picker shows them.
            pub const ALL: &[FolderGlyph] = &[$(FolderGlyph::$variant,)+];

            /// The glyph's file name, which is also how it is stored.
            pub fn name(self) -> &'static str {
                match self {
                    $(FolderGlyph::$variant => $name,)+
                }
            }

            /// What the glyph stands for, in the user's words.
            pub fn label(self) -> &'static str {
                match self {
                    $(FolderGlyph::$variant => $label,)+
                }
            }
        }
    };
}

folder_icons! {
    Read => "read", "Reading";
    Article => "article", "Articles";
    Code => "code", "Code";
    Write => "write", "Writing";
    Work => "work", "Work";
    Tag => "tag", "Deals";
    Bag => "bag", "Shopping";
    Plane => "plane", "Travel";
    Cup => "cup", "Food";
    Music => "music", "Music";
    Play => "play", "Video";
    Photo => "photo", "Photos";
    Bulb => "bulb", "Ideas";
    Heart => "heart", "Health";
    Home => "home", "Home";
    Star => "star", "Favorites";
    Clock => "clock", "Later";
    Flask => "flask", "Science";
    Globe => "globe", "Web";
    Person => "person", "People";
    Box => "box", "Archive";
    Inbox => "inbox", "Inbox";
    Leaf => "leaf", "Nature";
    Bolt => "bolt", "Quick";
    Lock => "lock", "Private";
    Pin => "pin", "Places";
    Chat => "chat", "Threads";
    Game => "game", "Games";
}

impl FolderGlyph {
    /// The glyph stored under `name`. `None` for a name this version doesn't know, so
    /// dropping a glyph from the set leaves its folders on the plain shape.
    pub fn from_name(name: &str) -> Option<FolderGlyph> {
        FolderGlyph::ALL
            .iter()
            .copied()
            .find(|glyph| glyph.name() == name)
    }
}

/// A folder's icon. Both kinds share one stored text value, told apart by whether it
/// names a glyph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FolderIcon {
    /// Tinted with the folder's color.
    Glyph(FolderGlyph),
    /// Drawn in its own colors.
    Emoji(Emoji),
}

impl FolderIcon {
    /// The icon stored as `value`. `None` for anything that is neither a known glyph nor
    /// one emoji, so the folder shows its plain shape.
    pub fn parse(value: &str) -> Option<FolderIcon> {
        FolderGlyph::from_name(value)
            .map(FolderIcon::Glyph)
            .or_else(|| Emoji::new(value).map(FolderIcon::Emoji))
    }

    /// The value it is stored and synced as.
    pub fn stored(&self) -> &str {
        match self {
            FolderIcon::Glyph(glyph) => glyph.name(),
            FolderIcon::Emoji(emoji) => emoji.as_str(),
        }
    }
}

impl From<FolderGlyph> for FolderIcon {
    fn from(glyph: FolderGlyph) -> FolderIcon {
        FolderIcon::Glyph(glyph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_values_read_back_as_glyphs_emoji_or_nothing() {
        let read = |value| FolderIcon::parse(value).map(|icon| icon.stored().to_owned());
        assert_eq!(read("read").as_deref(), Some("read"));
        assert!(matches!(
            FolderIcon::parse("read"),
            Some(FolderIcon::Glyph(_))
        ));
        let cases: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../sdk/fixtures/folder-icons.json"
        )))
        .unwrap();
        for emoji in cases["emoji"].as_array().unwrap() {
            let emoji = emoji.as_str().unwrap();
            assert_eq!(read(emoji).as_deref(), Some(emoji), "{emoji:?}");
        }
        for other in cases["invalid"].as_array().unwrap() {
            let other = other.as_str().unwrap();
            assert_eq!(read(other), None, "{other:?}");
        }
    }
}
