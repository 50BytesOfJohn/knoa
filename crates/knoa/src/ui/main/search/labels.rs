//! What each token shows: its mark and its label, in the user's language.

use gpui::{AnyElement, Hsla, IntoElement, Pixels, SharedString};
use knoa_core::{Comparison, FieldFilter, Filter, FolderIcon, Kind, Place};

use super::Slot;
use crate::{
    i18n::{self, t},
    state::{library::Library, search::Token},
    ui::{
        components::{Finish, folder_mark, kind_icon, svg_icon},
        main::{field_display, sidebar::place_icon},
        theme::Theme,
    },
};

/// What a token shows before its label.
pub(super) enum Mark {
    Icon(SharedString),
    Folder(Option<FolderIcon>, Hsla),
}

/// A token as drawn: what it is, its mark and its label.
pub(super) struct Chip {
    pub slot: Slot,
    pub mark: Mark,
    pub label: SharedString,
}

impl Mark {
    pub fn render(&self, size: Pixels, theme: &Theme) -> AnyElement {
        match self {
            Mark::Icon(path) => svg_icon(path.clone(), size, theme.muted).into_any_element(),
            Mark::Folder(glyph, color) => {
                folder_mark(glyph.as_ref(), *color, size, Finish::Line).into_any_element()
            }
        }
    }
}

/// The place as a token, or `None` for Everything, which has none.
pub(super) fn scope_chip(library: &Library, theme: &Theme) -> Option<Chip> {
    let place = library.place();
    let (name, mark) = match place {
        Place::Everything => return None,
        Place::Folder(id) => {
            let tree = library.folder_tree();
            let folder = tree.get(id)?;
            let color = theme.folder_color(folder.color);
            (tree.path_name(id), Mark::Folder(folder.icon.clone(), color))
        }
        place => (place_name(place), Mark::Icon(place_icon(place))),
    };
    Some(Chip {
        slot: Slot::Scope,
        mark,
        label: t!("main-token-scope", place = name).into(),
    })
}

fn place_name(place: Place) -> String {
    match place {
        Place::Gallery => t!("main-place-gallery"),
        Place::Events => t!("main-place-events"),
        Place::UpNext => t!("main-place-up-next"),
        Place::Unsorted => t!("main-place-unsorted"),
        Place::NotOrganized => t!("main-place-not-organized"),
        Place::SetAside => t!("main-place-set-aside"),
        Place::Trash => t!("main-place-trash"),
        Place::Everything | Place::Folder(_) => String::new(),
    }
}

pub(super) fn token_chip(index: usize, token: &Token) -> Chip {
    let (icon_name, label) = token_look(token);
    Chip {
        slot: Slot::Token(index),
        mark: Mark::Icon(format!("icons/{icon_name}.svg").into()),
        label: label.into(),
    }
}

/// A token's icon name and label, in the user's language.
pub(super) fn token_look(token: &Token) -> (&'static str, String) {
    match token {
        Token::Filter(Filter::Kind(kind)) => (kind_icon(Some(*kind)), i18n::kind_plural(*kind)),
        Token::Filter(Filter::Tag(tag)) => ("hash", tag.clone()),
        Token::Filter(Filter::Site(site)) => ("globe", site.clone()),
        Token::Filter(Filter::Field(field)) => ("text-align-start", field_label(field)),
        Token::Text(text) => ("search", t!("main-token-text", text = (text.as_str()))),
    }
}

/// "Director Christopher Nolan", "Runtime > 2h".
pub(super) fn field_label(field: &FieldFilter) -> String {
    let def = Kind::ALL
        .into_iter()
        .find_map(|kind| kind.field(&field.name));
    let Some(def) = def else {
        return field.name.clone();
    };
    let name = i18n::field_label(def);
    let value = field_display::display(def, &field.value);
    match field.op {
        Comparison::Matches => t!("main-token-field", field = name, value = value),
        Comparison::Greater => t!("main-token-field-more", field = name, value = value),
        Comparison::Less => t!("main-token-field-less", field = name, value = value),
    }
}
