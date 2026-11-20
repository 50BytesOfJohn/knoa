//! Controls that pick one of a few values: a segmented switch for two to four short
//! options seen at once, and a select field for a longer or per-row list.

use std::rc::Rc;

use gpui::{
    Anchor, App, Div, ElementId, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, StatefulInteractiveElement as _, Styled as _, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{
    Side,
    menu::{DropdownMenu as _, PopupMenuItem},
};

use crate::ui::{
    components::icon,
    main::menu_trigger::MenuTrigger,
    theme::{ROW_RADIUS, Theme},
};

/// One option: its value and what it says.
pub type Choice<T> = (T, String);

/// The options side by side in a pill, the chosen one raised.
pub fn segmented<T: Copy + PartialEq + 'static>(
    id: &'static str,
    options: Vec<Choice<T>>,
    current: Option<T>,
    disabled: bool,
    theme: &Theme,
    on_pick: impl Fn(T, &mut App) + 'static,
) -> Div {
    let on_pick = Rc::new(on_pick);
    let mut pill = div()
        .flex()
        .p(px(2.))
        .gap(px(2.))
        .rounded_full()
        .bg(theme.soft)
        .when(disabled, |s| s.opacity(0.5));
    for (index, (value, label)) in options.into_iter().enumerate() {
        let (text, hover) = (theme.text, theme.hover);
        let chosen = current == Some(value);
        let on_pick = on_pick.clone();
        pill = pill.child(
            div()
                .id((id, index))
                .h(px(26.))
                .px_3()
                .flex()
                .items_center()
                .rounded_full()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .map(|s| {
                    if chosen {
                        s.bg(theme.panel).text_color(text).shadow_sm()
                    } else {
                        s.text_color(theme.muted)
                    }
                })
                .when(!disabled && !chosen, |s| {
                    s.cursor_pointer()
                        .hover(move |s| s.bg(hover).text_color(text))
                        .on_click(move |_, _, cx| on_pick(value, cx))
                })
                .child(label),
        );
    }
    pill
}

/// A bordered field showing the chosen option, opening a menu of all of them.
pub fn select<T: Copy + PartialEq + 'static>(
    id: impl Into<ElementId>,
    options: Vec<Choice<T>>,
    current: Option<T>,
    disabled: bool,
    theme: &Theme,
    on_pick: impl Fn(T, &mut App) + 'static,
) -> gpui::AnyElement {
    let shown = options
        .iter()
        .find(|(value, _)| Some(*value) == current)
        .map(|(_, label)| label.clone())
        .unwrap_or_default();
    let field = div()
        .id(id.into())
        .h(px(30.))
        .min_w(px(160.))
        .px_2p5()
        .flex()
        .items_center()
        .gap_2()
        .rounded(ROW_RADIUS)
        .border_1()
        .border_color(theme.hairline)
        .text_sm()
        .child(div().flex_1().min_w_0().truncate().child(shown))
        .child(icon("chevrons-up-down", px(13.), theme.muted));
    if disabled {
        return field.opacity(0.5).into_any_element();
    }
    let on_pick = Rc::new(on_pick);
    MenuTrigger::new(field.cursor_pointer(), theme.soft)
        .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, _, _| {
            let menu = menu.check_side(Side::Right);
            options.iter().fold(menu, |menu, (value, label)| {
                let (value, on_pick) = (*value, on_pick.clone());
                menu.item(
                    PopupMenuItem::new(label.clone())
                        .checked(Some(value) == current)
                        .on_click(move |_, _, cx| {
                            if Some(value) != current {
                                on_pick(value, cx);
                            }
                        }),
                )
            })
        })
        .into_any_element()
}
