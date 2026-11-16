//! Refreshing untouched folder fields after the library changes.

use gpui::{Context, Window};
use gpui_component::slider::SliderValue;
use knoa_core::{Folder, FolderColor, FolderEdits};

use super::FolderEditor;

impl FolderEditor {
    pub(super) fn refresh(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let Some(baseline) = &self.existing else {
            return;
        };
        let tree = self.library.read(cx).folder_tree();
        // Compared by the parent it shows under, as the editor shows it.
        let Some(stored) = tree.get(baseline.id).map(|folder| Folder {
            parent: tree.parent(folder.id).map(|f| f.id),
            ..folder.clone()
        }) else {
            return;
        };
        let edited = Folder {
            name: self.name.read(cx).value().trim().to_owned(),
            description: self.description.read(cx).value().trim().to_owned(),
            auto_assign: self.auto_assign,
            color: self.color,
            icon: self.icon.clone(),
            parent: self.parent,
            ..baseline.clone()
        };
        let edits = FolderEdits::between(&baseline.clone().normalized(), &edited);
        let mut current = stored.clone().normalized();
        edits.apply_to(&mut current);
        self.existing = Some(stored);
        // Trimmed, so a refresh keeps the space or newline the user is still typing.
        if self.name.read(cx).value().trim() != current.name {
            self.name
                .update(cx, |input, cx| input.set_value(current.name, window, cx));
        }
        if self.description.read(cx).value().trim() != current.description {
            self.description.update(cx, |input, cx| {
                input.set_value(current.description, window, cx);
            });
        }
        if self.color != current.color
            && let FolderColor::Hue(hue) = current.color
        {
            self.hue.update(cx, |slider, cx| {
                slider.set_value(SliderValue::Single(f32::from(hue.degrees())), window, cx);
            });
        }
        self.color = current.color;
        self.icon = current.icon;
        self.auto_assign = current.auto_assign;
        self.parent = current.parent;
        cx.notify();
    }
}
