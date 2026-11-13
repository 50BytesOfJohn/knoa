//! Field-level changes to a folder, relative to an editor's baseline.

use crate::{Folder, FolderColor, FolderIcon};

/// Only fields the user changed; an absent field leaves the current value alone.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FolderEdits {
    pub name: Option<String>,
    pub description: Option<String>,
    pub auto_assign: Option<bool>,
    pub color: Option<FolderColor>,
    /// `Some(None)` clears the value; `None` leaves it alone.
    pub icon: Option<Option<FolderIcon>>,
}

impl FolderEdits {
    /// Captures changes relative to what the editor originally displayed.
    pub fn between(baseline: &Folder, edited: &Folder) -> Self {
        Self {
            name: (baseline.name != edited.name).then(|| edited.name.clone()),
            description: (baseline.description != edited.description)
                .then(|| edited.description.clone()),
            auto_assign: (baseline.auto_assign != edited.auto_assign).then_some(edited.auto_assign),
            color: (baseline.color != edited.color).then_some(edited.color),
            icon: (baseline.icon != edited.icon).then_some(edited.icon),
        }
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Applies intent to the current row, preserving every untouched field.
    pub fn apply_to(&self, current: &mut Folder) {
        if let Some(value) = &self.name {
            current.name.clone_from(value);
        }
        if let Some(value) = &self.description {
            current.description.clone_from(value);
        }
        if let Some(value) = self.auto_assign {
            current.auto_assign = value;
        }
        if let Some(value) = self.color {
            current.color = value;
        }
        if let Some(value) = self.icon {
            current.icon = value;
        }
    }
}

impl Folder {
    /// The folder as the editor shows it and reads it back, with trimmed text. An untouched
    /// editor compared against this has no edits.
    #[must_use]
    pub fn normalized(mut self) -> Self {
        self.name = self.name.trim().to_owned();
        self.description = self.description.trim().to_owned();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FolderId;

    #[test]
    fn an_untouched_editor_has_no_edits_against_untrimmed_storage() {
        let stored = Folder {
            id: FolderId(1),
            name: " Reading ".into(),
            description: "Long articles\n".into(),
            auto_assign: true,
            color: FolderColor::default(),
            icon: Some(FolderIcon::Read),
            count: 3,
        };
        // What the editor's inputs read back: each text field trimmed.
        let edited = Folder {
            name: stored.name.trim().to_owned(),
            description: stored.description.trim().to_owned(),
            ..stored.clone()
        };

        assert!(FolderEdits::between(&stored.normalized(), &edited).is_empty());
    }
}
