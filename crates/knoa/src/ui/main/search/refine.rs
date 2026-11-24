//! The refine row under the pill: one line of what splits the current results. At rest,
//! the kinds in view; while searching or filtered, "Narrow" with kinds and the tags
//! some but not all results have. What doesn't fit waits behind "More".

use std::{
    hash::{DefaultHasher, Hash as _, Hasher as _},
    rc::Rc,
};

use gpui::{
    Animation, AnimationExt as _, AnyElement, Context, Div, FontWeight, InteractiveElement,
    IntoElement, ParentElement as _, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use knoa_core::{Filter, Kind, Place};

use super::MainView;
use crate::{
    format,
    i18n::{self, t},
    state::{
        facets::ResultFacets,
        search::{Search, Token},
    },
    ui::{
        components::{tabular_figures, text_width},
        motion,
        theme::Theme,
    },
};

const CHIP_HEIGHT: f32 = 28.;
const CHIP_PADDING: f32 = 12.;
const CHIP_GAP: f32 = 6.;
const LABEL_COUNT_GAP: f32 = 5.;
const CHIP_TEXT: f32 = 12.;
/// More would be a wall of chips rather than a choice.
const MAX_CHIPS: usize = 30;
const MAX_NARROW_KINDS: usize = 3;

/// One choice in the row.
#[derive(Debug, Clone, PartialEq, Hash)]
pub(super) enum Choice {
    Kind(Kind, usize),
    Tag(String, usize),
}

impl Choice {
    fn label(&self) -> String {
        match self {
            Choice::Kind(kind, _) => i18n::kind_plural(*kind),
            Choice::Tag(tag, _) => format!("#{tag}"),
        }
    }

    fn count(&self) -> String {
        let (Choice::Kind(_, n) | Choice::Tag(_, n)) = self;
        format::number(*n as f64)
    }

    fn token(&self) -> Token {
        match self {
            Choice::Kind(kind, _) => Token::Filter(Filter::Kind(*kind)),
            Choice::Tag(tag, _) => Token::Filter(Filter::Tag(tag.clone())),
        }
    }
}

/// The row expanded by "More", until the search or place changes.
pub(super) type Expanded = Option<(Search, Place)>;

impl MainView {
    pub(super) fn render_refine(
        &mut self,
        width: f32,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let library = self.library.read(cx);
        let (search, place) = (library.search(), library.place());
        let choices = self
            .pill
            .refine_choices
            .get(search, place, || {
                choices(search, place, library.result_facets())
            })
            .clone();
        let narrowing = !search.is_empty();
        let clearable = !search.tokens.is_empty();
        let expanded = self
            .pill
            .refine_expanded
            .as_ref()
            .is_some_and(|(s, p)| s == search && *p == place);

        let mut fixed = 0.;
        if narrowing && !choices.is_empty() {
            fixed += measure(&t!("main-refine-narrow"), window) + CHIP_GAP;
        }
        if clearable {
            fixed += chip_width(&t!("main-refine-clear"), None, window) + CHIP_GAP;
        }
        let shown = if expanded {
            choices.len()
        } else {
            fitting(&choices, width - fixed, window)
        };

        let mut row = div()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(CHIP_GAP))
            .when(expanded, |s| {
                s.flex_wrap()
                    .p(px(6.))
                    .rounded(px(16.))
                    .bg(theme.panel)
                    .border_1()
                    .border_color(theme.hairline)
                    .shadow_lg()
            });
        if narrowing && !choices.is_empty() {
            row = row.child(
                div()
                    .text_size(px(11.5))
                    .text_color(theme.faint)
                    .child(t!("main-refine-narrow")),
            );
        }
        for (index, choice) in choices.iter().take(shown).enumerate() {
            row = row.child(self.choice_chip(index, choice, &theme, cx));
        }
        if shown < choices.len() {
            row = row.child(self.more_chip(&theme, cx));
        }
        if clearable {
            row = row.child(self.clear_chip(&theme, cx));
        }
        let mut hasher = DefaultHasher::new();
        choices.hash(&mut hasher);
        div()
            .id("refine-row")
            .w(px(width))
            .flex()
            .justify_center()
            .when(expanded, InteractiveElement::occlude)
            .child(row)
            .with_animation(
                ("refine-fade", hasher.finish() as usize),
                Animation::new(motion::CHIP_FADE),
                Styled::opacity,
            )
    }

    fn choice_chip(
        &self,
        index: usize,
        choice: &Choice,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let token = choice.token();
        chip(("refine", index), theme)
            .bg(theme.panel.opacity(0.7))
            .hover(|s| s.bg(theme.panel))
            .child(choice.label())
            .child(
                div()
                    .text_color(theme.faint)
                    .font_features(tabular_figures())
                    .child(choice.count()),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                let mut search = this.library.read(cx).search().clone();
                search.add(token.clone());
                this.set_search(search, window, cx);
            }))
            .into_any_element()
    }

    fn more_chip(&self, theme: &Theme, cx: &mut Context<'_, MainView>) -> AnyElement {
        chip("refine-more", theme)
            .hover(|s| s.bg(theme.panel.opacity(0.7)))
            .child(t!("main-refine-more"))
            .on_click(cx.listener(|this, _, _, cx| {
                let library = this.library.read(cx);
                this.pill.refine_expanded = Some((library.search().clone(), library.place()));
                cx.notify();
            }))
            .into_any_element()
    }

    /// Takes away every token and the text; the scope stays.
    fn clear_chip(&self, theme: &Theme, cx: &mut Context<'_, MainView>) -> AnyElement {
        chip("refine-clear", theme)
            .hover(|s| s.bg(theme.panel.opacity(0.7)))
            .child(t!("main-refine-clear"))
            .on_click(cx.listener(|this, _, window, cx| this.clear_search(window, cx)))
            .into_any_element()
    }
}

/// What splits the results. At rest, the kinds in view, or the tags in the Gallery and
/// Events, which hold one or a few kinds. Filtered, kinds while no kind is applied, then
/// tags some but not all results have. Never a choice at 0, never one already applied,
/// never one that narrows nothing.
fn choices(search: &Search, place: Place, results: &ResultFacets) -> Rc<[Choice]> {
    let splitting_tags = results
        .tags
        .iter()
        .filter(|(tag, n)| *n > 0 && *n < results.total && !search.has_tag(tag))
        .map(|(tag, n)| Choice::Tag(tag.clone(), *n));
    let kinds = results.kinds.iter().map(|&(kind, n)| Choice::Kind(kind, n));
    let tags_first = matches!(place, Place::Gallery | Place::Events);
    let one_kind_for_all = matches!(results.kinds.as_slice(), [(_, n)] if *n == results.total);
    let several_kinds = results.kinds.len() > 1;
    let choices: Box<dyn Iterator<Item = Choice>> = if search.is_empty() {
        if tags_first {
            Box::new(splitting_tags)
        } else if one_kind_for_all {
            Box::new(std::iter::empty())
        } else {
            Box::new(kinds)
        }
    } else if search.kind().is_none() && several_kinds && !tags_first {
        Box::new(kinds.take(MAX_NARROW_KINDS).chain(splitting_tags))
    } else {
        Box::new(splitting_tags)
    };
    choices.take(MAX_CHIPS).collect()
}

/// How many choices fit on one line in `room`, keeping space for "More" when not all do.
fn fitting(choices: &[Choice], room: f32, window: &Window) -> usize {
    let widths: Vec<f32> = choices
        .iter()
        .map(|c| chip_width(&c.label(), Some(&c.count()), window) + CHIP_GAP)
        .collect();
    if widths.iter().sum::<f32>() <= room {
        return choices.len();
    }
    let room = room - chip_width(&t!("main-refine-more"), None, window) - CHIP_GAP;
    let mut used = 0.;
    widths
        .iter()
        .take_while(|w| {
            used += *w;
            used <= room
        })
        .count()
}

fn chip_width(label: &str, count: Option<&str>, window: &Window) -> f32 {
    let count = count.map_or(0., |count| LABEL_COUNT_GAP + measure(count, window));
    CHIP_PADDING * 2. + measure(label, window) + count
}

fn measure(text: &str, window: &Window) -> f32 {
    f32::from(text_width(text, px(CHIP_TEXT), FontWeight::MEDIUM, window))
}

fn chip(id: impl Into<gpui::ElementId>, theme: &Theme) -> gpui::Stateful<Div> {
    div()
        .id(id)
        .flex_none()
        .h(px(CHIP_HEIGHT))
        .px(px(CHIP_PADDING))
        .flex()
        .items_center()
        .gap(px(LABEL_COUNT_GAP))
        .rounded_full()
        .text_size(px(CHIP_TEXT))
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.muted)
        .cursor_pointer()
}
