//! Adding a link by hand to the current workspace. It skips the workspace's guideline:
//! the user chose it.

use gpui::{
    AppContext as _, Context, Entity, FontWeight, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Subscription, WeakEntity, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::input::{Input, InputEvent, InputState};
use knoa_core::{Place, parse_link};

use super::MainView;
use crate::{
    services::organizer::Organizer,
    state::library::Library,
    ui::{
        components::{Tone, button},
        theme::{SERIF, Theme},
    },
};

pub struct AddLink {
    url: Entity<InputState>,
    error: Option<&'static str>,
    library: Entity<Library>,
    main: WeakEntity<MainView>,
    _enter: Subscription,
}

impl AddLink {
    pub fn new(
        library: Entity<Library>,
        main: WeakEntity<MainView>,
        window: &mut Window,
        cx: &mut Context<'_, AddLink>,
    ) -> AddLink {
        let url = cx.new(|cx| InputState::new(window, cx).placeholder("https://…"));
        // A link on the clipboard is most likely the one to add.
        let pasted = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .filter(|text| parse_link(text).is_some());
        url.update(cx, |input, cx| {
            if let Some(text) = pasted {
                input.set_value(text.trim().to_owned(), window, cx);
            }
            input.focus(window, cx);
        });
        let enter = cx.subscribe_in(&url, window, |this, _, event: &InputEvent, window, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                this.add(window, cx);
            }
        });
        AddLink {
            url,
            error: None,
            library,
            main,
            _enter: enter,
        }
    }

    fn add(&mut self, window: &mut Window, cx: &mut Context<'_, AddLink>) {
        let Some(url) = parse_link(&self.url.read(cx).value()) else {
            self.error = Some("That doesn't look like a web address.");
            cx.notify();
            return;
        };
        let task = self
            .library
            .update(cx, |library, cx| library.add_link(url, cx));
        let main = self.main.clone();
        let window_handle = window.window_handle();
        cx.spawn(async move |_, cx| match task.await {
            Ok(added) => {
                let _ = cx.update_window(window_handle, |_, window, cx| {
                    Organizer::wake(cx);
                    let _ = main.update(cx, |main, cx| {
                        main.set_place(Place::Everything, window, cx);
                        main.select(added.id, cx);
                    });
                });
            }
            Err(err) => tracing::error!("adding a link failed: {err}"),
        })
        .detach();
        self.close(window, cx);
    }

    fn close(&self, window: &mut Window, cx: &mut Context<'_, AddLink>) {
        let _ = self
            .main
            .update(cx, |main, cx| main.close_overlay(window, cx));
    }
}

impl Render for AddLink {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let workspace = self
            .library
            .read(cx)
            .workspace()
            .map(|w| w.name.clone())
            .unwrap_or_default();
        div()
            .w(px(420.))
            .p_5()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .font_family(SERIF)
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(format!("Add a link to {workspace}")),
            )
            .child(Input::new(&self.url))
            .when_some(self.error, |s, error| {
                s.child(div().text_sm().text_color(theme.danger).child(error))
            })
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().flex_1())
                    .child(
                        button("cancel-link", "Cancel", Tone::Ghost, cx)
                            .on_click(cx.listener(|this, _, window, cx| this.close(window, cx))),
                    )
                    .child(
                        button("add-link", "Add", Tone::Primary, cx)
                            .on_click(cx.listener(|this, _, window, cx| this.add(window, cx))),
                    ),
            )
    }
}
