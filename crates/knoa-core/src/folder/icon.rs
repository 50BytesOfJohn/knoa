//! The glyphs a folder can show instead of the folder shape.

macro_rules! folder_icons {
    ($($variant:ident => $name:literal, $label:literal;)+) => {
        /// A glyph from Knoa's own icon set.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum FolderIcon {
            $($variant,)+
        }

        impl FolderIcon {
            /// Every glyph, in the order the picker shows them.
            pub const ALL: &[FolderIcon] = &[$(FolderIcon::$variant,)+];

            /// The glyph's file name, which is also how it is stored.
            pub fn name(self) -> &'static str {
                match self {
                    $(FolderIcon::$variant => $name,)+
                }
            }

            /// What the glyph stands for, in the user's words.
            pub fn label(self) -> &'static str {
                match self {
                    $(FolderIcon::$variant => $label,)+
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

impl FolderIcon {
    /// The glyph stored under `name`. `None` for a name this version doesn't know, so
    /// dropping a glyph from the set leaves its folders on the plain shape.
    pub fn from_name(name: &str) -> Option<FolderIcon> {
        FolderIcon::ALL
            .iter()
            .copied()
            .find(|icon| icon.name() == name)
    }
}
