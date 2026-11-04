//! Creating and editing a folder: name, what belongs in it, auto-assign and colour.

use gpui::{
    AppContext as _, Context, Entity, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _, WeakEntity, Window,
    div, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    input::{Input, InputState, Textarea, TextareaState},
    switch::Switch,
};
use knoa_core::{Folder, FolderColor, FolderId};

use super::MainView;
use crate::{
    state::library::Library,
    ui::{
        components::{Tone, button},
        theme::Theme,
    },
};

pub struct FolderEditor {
    existing: Option<Folder>,
    name: Entity<InputState>,
    description: Entity<TextareaState>,
    auto_assign: bool,
    color: u8,
    library: Entity<Library>,
    main: WeakEntity<MainView>,
}

impl FolderEditor {
    pub fn new(
        existing: Option<Folder>,
        library: Entity<Library>,
        main: WeakEntity<MainView>,
        window: &mut Window,
        cx: &mut Context<'_, FolderEditor>,
    ) -> FolderEditor {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("Folder name"));
        let description = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("What belongs here? The AI uses this to file bookmarks.")
        });
        if let Some(folder) = &existing {
            name.update(cx, |input, cx| {
                input.set_value(folder.name.clone(), window, cx);
            });
            description.update(cx, |input, cx| {
                input.set_value(folder.description.clone(), window, cx);
            });
        }
        name.update(cx, |input, cx| input.focus(window, cx));
        let next_color = library.read(cx).folders().len() as u8;
        FolderEditor {
            auto_assign: existing.as_ref().is_none_or(|f| f.auto_assign),
            color: existing.as_ref().map_or(next_color % 8, |f| f.color.0),
            existing,
            name,
            description,
            library,
            main,
        }
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<'_, FolderEditor>) {
        let name = self.name.read(cx).value().trim().to_owned();
        if name.is_empty() {
            return;
        }
        let description = self.description.read(cx).value().trim().to_owned();
        let (color, auto_assign) = (FolderColor(self.color), self.auto_assign);
        match &self.existing {
            Some(folder) => {
                let folder = Folder {
                    name,
                    description,
                    auto_assign,
                    color,
                    ..folder.clone()
                };
                self.library.update(cx, |library, cx| {
                    library.change(cx, move |store| store.update_folder(&folder));
                });
            }
            None => self.library.update(cx, |library, cx| {
                library.change(cx, move |store| {
                    store
                        .create_folder(&name, &description, auto_assign, color)
                        .map(drop)
                });
            }),
        }
        self.close(window, cx);
    }

    fn delete(&mut self, id: FolderId, window: &mut Window, cx: &mut Context<'_, FolderEditor>) {
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.delete_folder(id));
        });
        self.close(window, cx);
    }

    fn close(&self, window: &mut Window, cx: &mut Context<'_, FolderEditor>) {
        let _ = self
            .main
            .update(cx, |main, cx| main.close_overlay(window, cx));
    }
}

impl Render for FolderEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let mut swatches = div().flex().gap_2();
        for index in 0..8u8 {
            let selected = index == self.color;
            swatches = swatches.child(
                div()
                    .id(("swatch", usize::from(index)))
                    .size(px(22.))
                    .rounded_full()
                    .bg(theme.folder_color(index))
                    .cursor_pointer()
                    .when(selected, |s| s.border_2().border_color(theme.text))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.color = index;
                        cx.notify();
                    })),
            );
        }
        let title = if self.existing.is_some() {
            "Edit folder"
        } else {
            "New folder"
        };
        let existing = self.existing.as_ref().map(|f| f.id);
        div()
            .w(px(380.))
            .p_5()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .font_family(crate::ui::theme::SERIF)
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title),
            )
            .child(Input::new(&self.name))
            .child(Textarea::new(&self.description).h(px(84.)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(div().text_sm().child("Let the AI add bookmarks"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted)
                                    .child("New bookmarks that fit go here on their own."),
                            ),
                    )
                    .child(
                        Switch::new("auto-assign")
                            .checked(self.auto_assign)
                            .on_click(cx.listener(|this, on: &bool, _, cx| {
                                this.auto_assign = *on;
                                cx.notify();
                            })),
                    ),
            )
            .child(swatches)
            .child(
                div()
                    .flex()
                    .gap_2()
                    .when_some(existing, |s, id| {
                        s.child(button("delete-folder", "Delete", Tone::Ghost, cx).on_click(
                            cx.listener(move |this, _, window, cx| this.delete(id, window, cx)),
                        ))
                    })
                    .child(div().flex_1())
                    .child(
                        button("cancel-folder", "Cancel", Tone::Ghost, cx)
                            .on_click(cx.listener(|this, _, window, cx| this.close(window, cx))),
                    )
                    .child(
                        button("save-folder", "Save", Tone::Primary, cx)
                            .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                    ),
            )
    }
}
