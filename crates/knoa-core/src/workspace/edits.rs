//! Field-level changes to a workspace, relative to an editor's baseline.

use crate::{Ink, ListedTag, Workspace};

/// Only fields the user changed; an absent field leaves the current value alone.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WorkspaceEdits {
    pub name: Option<String>,
    pub color: Option<Ink>,
    pub browsers: Option<Vec<String>>,
    /// `Some(None)` clears the value; `None` leaves it alone.
    pub opens_in: Option<Option<String>>,
    pub instructions: Option<String>,
    pub screening: Option<bool>,
    pub guideline: Option<String>,
    pub tags: Option<Vec<ListedTag>>,
    pub only_listed_tags: Option<bool>,
}

impl WorkspaceEdits {
    /// Captures changes relative to what the editor originally displayed.
    pub fn between(baseline: &Workspace, edited: &Workspace) -> Self {
        Self {
            name: (baseline.name != edited.name).then(|| edited.name.clone()),
            color: (baseline.color != edited.color).then_some(edited.color),
            browsers: (baseline.browsers != edited.browsers).then(|| edited.browsers.clone()),
            opens_in: (baseline.opens_in != edited.opens_in).then(|| edited.opens_in.clone()),
            instructions: (baseline.instructions != edited.instructions)
                .then(|| edited.instructions.clone()),
            screening: (baseline.screening.enabled != edited.screening.enabled)
                .then_some(edited.screening.enabled),
            guideline: (baseline.screening.guideline != edited.screening.guideline)
                .then(|| edited.screening.guideline.clone()),
            tags: (baseline.tag_list.tags != edited.tag_list.tags)
                .then(|| edited.tag_list.tags.clone()),
            only_listed_tags: (baseline.tag_list.only_these != edited.tag_list.only_these)
                .then_some(edited.tag_list.only_these),
        }
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Applies intent to the current row, preserving every untouched field.
    pub fn apply_to(&self, current: &mut Workspace) {
        if let Some(value) = &self.name {
            current.name.clone_from(value);
        }
        if let Some(value) = self.color {
            current.color = value;
        }
        if let Some(value) = &self.browsers {
            current.browsers.clone_from(value);
        }
        if let Some(value) = &self.opens_in {
            current.opens_in.clone_from(value);
        }
        if let Some(value) = &self.instructions {
            current.instructions.clone_from(value);
        }
        if let Some(value) = self.screening {
            current.screening.enabled = value;
        }
        if let Some(value) = &self.guideline {
            current.screening.guideline.clone_from(value);
        }
        if let Some(value) = &self.tags {
            current.tag_list.tags.clone_from(value);
        }
        if let Some(value) = self.only_listed_tags {
            current.tag_list.only_these = value;
        }
    }
}

impl Workspace {
    /// The workspace as the editor shows it and reads it back: trimmed text and tag hints,
    /// tags sorted by name. An untouched editor compared against this has no edits.
    #[must_use]
    pub fn normalized(mut self) -> Self {
        self.name = self.name.trim().to_owned();
        self.instructions = self.instructions.trim().to_owned();
        self.screening.guideline = self.screening.guideline.trim().to_owned();
        for tag in &mut self.tag_list.tags {
            tag.description = tag.description.trim().to_owned();
        }
        self.tag_list.tags.sort_by(|a, b| a.name.cmp(&b.name));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Screening, TagList, WorkspaceId};

    #[test]
    fn an_untouched_editor_has_no_edits_against_untrimmed_storage() {
        let tag = |name: &str, description: &str| ListedTag {
            name: name.into(),
            description: description.into(),
        };
        let stored = Workspace {
            id: WorkspaceId(1),
            name: " Work ".into(),
            color: Ink::default(),
            browsers: vec!["helium:Default".into()],
            opens_in: None,
            instructions: "Use short tags\n".into(),
            screening: Screening {
                enabled: true,
                guideline: " Only work ".into(),
            },
            tag_list: TagList {
                tags: vec![tag("rust", " Language "), tag("go", "")],
                only_these: true,
            },
        };
        // What the editor's inputs read back: each field trimmed, tag rows sorted by name.
        let mut edited = stored.clone();
        edited.name = stored.name.trim().to_owned();
        edited.instructions = stored.instructions.trim().to_owned();
        edited.screening.guideline = stored.screening.guideline.trim().to_owned();
        edited.tag_list.tags = vec![tag("go", ""), tag("rust", "Language")];

        assert!(WorkspaceEdits::between(&stored.normalized(), &edited).is_empty());
    }
}
