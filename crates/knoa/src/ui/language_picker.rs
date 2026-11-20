//! A menu of the languages Knoa speaks, each in its own name, so anyone finds theirs.

use std::rc::Rc;

use gpui::{
    Anchor, App, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _, div, px,
};
use gpui_component::{
    Side,
    menu::{DropdownMenu as _, PopupMenuItem},
};
use knoa_core::Language;

use crate::ui::{
    components::icon,
    main::menu_trigger::MenuTrigger,
    theme::{ROW_RADIUS, Theme},
};

/// How the picker sits among what's around it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Look {
    /// A bordered field, as a setting's control.
    Field,
    /// A quiet line of text with a globe, for a screen that's about something else.
    Compact,
}

pub fn language_picker(
    id: &'static str,
    current: Language,
    look: Look,
    theme: &Theme,
    on_pick: impl Fn(Language, &mut App) + 'static,
) -> impl IntoElement {
    let base = div()
        .id(id)
        .h(px(30.))
        .px_2p5()
        .flex()
        .items_center()
        .gap_2()
        .rounded(ROW_RADIUS)
        .text_sm()
        .cursor_pointer();
    let base = match look {
        Look::Field => base
            .min_w(px(160.))
            .border_1()
            .border_color(theme.hairline)
            .child(div().flex_1().child(current.native_name())),
        Look::Compact => base
            .text_color(theme.muted)
            .hover(|s| s.bg(theme.soft))
            .child(icon("globe", px(14.), theme.muted))
            .child(current.native_name()),
    }
    .child(icon("chevrons-up-down", px(13.), theme.muted));
    let on_pick = Rc::new(on_pick);
    let anchor = match look {
        Look::Field => Anchor::TopRight,
        Look::Compact => Anchor::TopCenter,
    };
    MenuTrigger::new(base, theme.soft).dropdown_menu_with_anchor(anchor, move |menu, _, _| {
        let menu = menu.check_side(Side::Right);
        Language::ALL.into_iter().fold(menu, |menu, language| {
            let on_pick = on_pick.clone();
            menu.item(
                PopupMenuItem::new(language.native_name())
                    .checked(language == current)
                    .on_click(move |_, _, cx| {
                        if language != current {
                            on_pick(language, cx);
                        }
                    }),
            )
        })
    })
}
