//! Knoa's handwritten wordmark, "Knoa.", drawn stroke by stroke the way a pen writes it.
//!
//! The strokes are the same as in `assets/brand/wordmark.svg`; change both together.

use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, Bounds, Div, ElementId, Hsla, IntoElement, ParentElement as _,
    PathBuilder, PathStyle, Pixels, Point, StrokeOptions, Styled as _, Window, canvas, div,
    ease_in_out, fill, point, px, rgb, size,
};
use lyon_tessellation::{LineCap, LineJoin};

/// The wordmark at rest, fully written.
pub fn wordmark(height: Pixels, ink: Hsla) -> Div {
    let shape = Shape::new(height);
    frame(height, &shape).child(drawing(shape, ink, None))
}

/// The wordmark writing itself once, when it first appears.
pub fn written_wordmark(id: impl Into<ElementId>, height: Pixels, ink: Hsla) -> impl IntoElement {
    let shape = Shape::new(height);
    let total = Timeline::new().total;
    frame(height, &shape).with_animation(
        id,
        Animation::new(Duration::from_secs_f32(total / 1000.)),
        move |el, delta| el.child(drawing(shape, ink, Some(delta * total))),
    )
}

/// The dot keeps its colour in every theme: it's part of the logo, not of the UI.
const DOT_COLOR: u32 = 0x00e0_573c;

/// A pen stroke in the design's units: where it starts, then cubic segments as
/// `[control_a.x, control_a.y, control_b.x, control_b.y, to.x, to.y]`.
struct Stroke {
    start: [f32; 2],
    curves: &'static [[f32; 6]],
}

/// The K's stem, then the K's arm and leg running on through "noa" without a lift.
const STROKES: [Stroke; 2] = [
    Stroke {
        start: [64., 32.],
        curves: &[[62., 62., 57., 95., 52., 122.]],
    },
    Stroke {
        start: [100., 32.],
        curves: &[
            [86., 50., 72., 66., 57., 76.],
            [70., 80., 80., 100., 90., 115.],
            [95., 122., 101., 121., 106., 114.],
            [110., 100., 113., 90., 116., 82.],
            [116., 96., 114., 108., 112., 120.],
            [116., 96., 126., 82., 136., 82.],
            [146., 82., 146., 96., 144., 108.],
            [143., 116., 148., 120., 156., 116.],
            [164., 112., 172., 92., 188., 82.],
            [174., 80., 168., 98., 170., 110.],
            [172., 122., 188., 124., 196., 112.],
            [203., 100., 200., 82., 188., 82.],
            [196., 86., 214., 86., 236., 82.],
            [220., 80., 212., 96., 214., 108.],
            [216., 122., 234., 122., 242., 102.],
            [244., 94., 246., 86., 246., 82.],
            [244., 98., 242., 114., 248., 119.],
            [251., 123., 257., 121., 261., 117.],
        ],
    },
];

const LEFT: f32 = 52.;
const TOP: f32 = 32.;
const BASELINE: f32 = 122.;
const TAIL_END_X: f32 = 261.;
const DOT_Y: f32 = 118.;

/// The wordmark's geometry at one display height.
#[derive(Clone, Copy)]
struct Shape {
    /// Pen width in design units; the dot's radius matches it.
    weight: f32,
    origin: [f32; 2],
    size: [f32; 2],
    dot: [f32; 2],
}

impl Shape {
    fn new(height: Pixels) -> Shape {
        // Small marks get a heavier pen, or the strokes thin out to hairlines.
        let weight = (5. + (80. - f32::from(height)) / 15.).clamp(5., 9.);
        let half = weight / 2.;
        let dot = [TAIL_END_X + half + weight * 0.6 + weight, DOT_Y];
        let origin = [LEFT - half, TOP - half];
        let right = dot[0] + weight;
        let bottom = (BASELINE + half).max(DOT_Y + weight);
        Shape {
            weight,
            origin,
            size: [right - origin[0], bottom - origin[1]],
            dot,
        }
    }

    fn aspect(&self) -> f32 {
        self.size[0] / self.size[1]
    }
}

fn frame(height: Pixels, shape: &Shape) -> Div {
    div().flex_none().h(height).w(height * shape.aspect())
}

fn drawing(shape: Shape, ink: Hsla, elapsed: Option<f32>) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| paint(&shape, bounds, ink, elapsed, window),
    )
    .size_full()
}

/// Paints the wordmark as it looks `elapsed` ms into writing it, or finished for `None`.
fn paint(
    shape: &Shape,
    bounds: Bounds<Pixels>,
    ink: Hsla,
    elapsed: Option<f32>,
    window: &mut Window,
) {
    let scale = f32::from(bounds.size.height) / shape.size[1];
    let at = |[x, y]: [f32; 2]| {
        point(
            bounds.origin.x + px((x - shape.origin[0]) * scale),
            bounds.origin.y + px((y - shape.origin[1]) * scale),
        )
    };
    let timeline = Timeline::new();

    for (stroke, timing) in STROKES.iter().zip(timeline.strokes) {
        let written = elapsed.map_or(1., |t| timing.progress(t));
        if written > 0. {
            paint_stroke(
                stroke,
                written * timing.length * scale,
                shape.weight * scale,
                &at,
                ink,
                window,
            );
        }
    }

    let (opacity, lift) =
        elapsed.map_or((1., 0.), |t| dot_drop((t - timeline.dot_start) / DOT_DROP));
    if opacity > 0. {
        let radius = px(shape.weight * scale);
        let center = at([shape.dot[0], shape.dot[1] + lift]);
        let bounds = Bounds::new(
            point(center.x - radius, center.y - radius),
            size(radius * 2., radius * 2.),
        );
        let color: Hsla = rgb(DOT_COLOR).into();
        window.paint_quad(fill(bounds, color.opacity(opacity)).corner_radii(radius));
    }
}

/// Paints the first `length` pixels of a stroke.
fn paint_stroke(
    stroke: &Stroke,
    length: f32,
    width: f32,
    at: &impl Fn([f32; 2]) -> Point<Pixels>,
    ink: Hsla,
    window: &mut Window,
) {
    let options = StrokeOptions::default()
        .with_line_width(width)
        .with_line_cap(LineCap::Round)
        .with_line_join(LineJoin::Round);
    // A dash as long as the written part, then a gap longer than the stroke, is how an
    // SVG pen animation works; the path builder supports the same trick.
    let mut path = PathBuilder::default()
        .with_style(PathStyle::Stroke(options))
        .dash_array(&[px(length), px(length.max(1.) * 4.)]);
    path.move_to(at(stroke.start));
    for c in stroke.curves {
        path.cubic_bezier_to(at([c[4], c[5]]), at([c[0], c[1]]), at([c[2], c[3]]));
    }
    // A stroke only a few pixels in can fail to tessellate; it simply appears next frame.
    if let Ok(path) = path.build() {
        window.paint_path(path, ink);
    }
}

/// Pen speed in design units per millisecond: about the pace of unhurried handwriting.
const PEN_SPEED: f32 = 0.32;
const PEN_LIFT: f32 = 140.;
const LEAD_IN: f32 = 120.;
const DOT_DROP: f32 = 520.;

#[derive(Clone, Copy)]
struct StrokeTiming {
    start: f32,
    duration: f32,
    length: f32,
}

impl StrokeTiming {
    fn progress(self, elapsed: f32) -> f32 {
        ease_in_out(((elapsed - self.start) / self.duration).clamp(0., 1.))
    }
}

/// When each stroke and the dot happen, in milliseconds: strokes take time in proportion
/// to their length, with a pen lift between them, as a hand would write.
struct Timeline {
    strokes: [StrokeTiming; 2],
    dot_start: f32,
    total: f32,
}

impl Timeline {
    fn new() -> Timeline {
        let mut clock = LEAD_IN;
        let strokes = STROKES.each_ref().map(|stroke| {
            let length = stroke_length(stroke);
            let timing = StrokeTiming {
                start: clock,
                duration: length / PEN_SPEED,
                length,
            };
            clock += timing.duration + PEN_LIFT;
            timing
        });
        Timeline {
            strokes,
            dot_start: clock,
            total: clock + DOT_DROP,
        }
    }
}

fn stroke_length(stroke: &Stroke) -> f32 {
    const STEPS: usize = 16;
    let mut from = stroke.start;
    let mut length = 0.;
    for c in stroke.curves {
        let mut prev = from;
        for step in 1..=STEPS {
            let next = cubic_point(from, c, step as f32 / STEPS as f32);
            length += (next[0] - prev[0]).hypot(next[1] - prev[1]);
            prev = next;
        }
        from = [c[4], c[5]];
    }
    length
}

/// The point at `t` along the cubic from `start` through `curve`, by its Bernstein weights.
fn cubic_point(start: [f32; 2], curve: &[f32; 6], t: f32) -> [f32; 2] {
    let rest = 1. - t;
    let weights = [
        rest * rest * rest,
        3. * rest * rest * t,
        3. * rest * t * t,
        t * t * t,
    ];
    let along = |axis: usize| {
        weights[0] * start[axis]
            + weights[1] * curve[axis]
            + weights[2] * curve[2 + axis]
            + weights[3] * curve[4 + axis]
    };
    [along(0), along(1)]
}

/// The dot falls in, overshoots and settles: its opacity and vertical offset (in design
/// units) at `t` from 0 to 1.
fn dot_drop(t: f32) -> (f32, f32) {
    match t {
        t if t <= 0. => (0., -18.),
        t if t < 0.55 => {
            let k = t / 0.55;
            (k, -18. * (1. - k) * (1. - k))
        }
        t if t < 0.75 => (1., -4. * ease_in_out((t - 0.55) / 0.2)),
        t if t < 1. => (1., -4. * (1. - ease_in_out((t - 0.75) / 0.25))),
        _ => (1., 0.),
    }
}
