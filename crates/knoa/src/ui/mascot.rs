//! The AI mascot: the logo's full stop come alive. A dot pressed by a real nib, glancing
//! aside, with a face from `FACE_MIN` up and a plain dot below it.

use std::{f32::consts::TAU, time::Duration};

use gpui::{
    Animation, AnimationExt as _, AnyElement, Bounds, Div, ElementId, Hsla, IntoElement,
    ParentElement as _, PathBuilder, PathStyle, Pixels, Point, StrokeOptions, Styled as _, Window,
    canvas, div, fill, point, px, size,
};
use lyon_tessellation::{LineCap, LineJoin};

use crate::services::organizer::AiStatus;

/// What the AI is up to, which sets the face and the motion.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Idle,
    Organizing,
    Happy,
    Worried,
    Asleep,
}

/// The AI's mark beside text and in chips: the mascot's body without its face.
pub fn ai_mark(size: Pixels, ink: Hsla) -> Div {
    div()
        .size(size)
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .child(div().size(size * 0.72).rounded_full().bg(ink))
}

/// The mascot in a mood. Its face is cut out of the body, so it takes the colour of the
/// `surface` behind it.
pub fn mascot(
    id: impl Into<ElementId>,
    size: Pixels,
    mood: Mood,
    ink: Hsla,
    surface: Hsla,
) -> AnyElement {
    let frame = div().flex_none().size(size);
    let Some((duration, looped)) = mood.motion() else {
        return frame
            .child(drawing(mood, ink, surface, 0.))
            .into_any_element();
    };
    let seconds = duration.as_secs_f32();
    // Its own view re-renders on every frame it moves, so it moves at film rate, not 120 Hz.
    let mut animation = Animation::new(duration).with_max_fps(30.);
    if looped {
        animation = animation.repeat();
    }
    frame
        .with_animation(id, animation, move |el, delta| {
            el.child(drawing(mood, ink, surface, delta * seconds))
        })
        .into_any_element()
}

impl Mood {
    /// How the mascot looks while the AI is in `status` with `organizing` bookmarks queued.
    pub fn of(status: &AiStatus, organizing: usize) -> Mood {
        match status {
            AiStatus::Off => Mood::Asleep,
            AiStatus::NeedsAttention(_) => Mood::Worried,
            AiStatus::Ready if organizing > 0 => Mood::Organizing,
            AiStatus::Ready => Mood::Idle,
        }
    }

    /// How long its motion lasts and whether it loops. Calm moods stand still: a mascot
    /// that fidgets all day redraws the window all day.
    fn motion(self) -> Option<(Duration, bool)> {
        match self {
            Mood::Organizing => Some((Duration::from_millis(6400), true)),
            Mood::Happy => Some((Duration::from_millis(4500), false)),
            Mood::Worried => Some((Duration::from_millis(2600), false)),
            Mood::Idle | Mood::Asleep => None,
        }
    }
}

/// Below this the face is noise, so the mascot is only its dot.
const FACE_MIN: Pixels = px(18.);

// The design is drawn in units on a 48 grid; the visible box is 40 units from (4, 7).
const BOX_ORIGIN: [f32; 2] = [4., 7.];
const BOX_SIZE: f32 = 40.;
const CENTER: [f32; 2] = [24., 27.];
const RADIUS: f32 = 17.;
/// Where it stands: squash, stretch and tilt pivot here.
const FEET: [f32; 2] = [24., 44.];
const EYES: [[f32; 2]; 2] = [[20.6, 27.6], [30.2, 27.6]];
const EYE_RADIUS: f32 = 2.;
/// Off-centre, under the eyes' gaze.
const MOUTH: [f32; 2] = [26.8, 31.2];
const LINE: f32 = 1.8;

fn drawing(mood: Mood, ink: Hsla, surface: Hsla, seconds: f32) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let pen = Pen::new(bounds, pose(mood, seconds), ink, surface);
            pen.body(window);
            if bounds.size.height >= FACE_MIN {
                face(mood, &pen, window);
            }
            extras(mood, seconds, &pen, window);
        },
    )
    .size_full()
}

/// The body's motion at one moment.
#[derive(Clone, Copy)]
struct Pose {
    lift: f32,
    sway: f32,
    squash: [f32; 2],
    tilt: f32,
    look: [f32; 2],
    /// How open the eyes are, from 0 to 1.
    open: f32,
    /// Phase of the slow wobble along the ink's edge.
    ripple: f32,
}

const REST: Pose = Pose {
    lift: 0.,
    sway: 0.,
    squash: [1., 1.],
    tilt: 0.,
    look: [0., 0.],
    open: 1.,
    ripple: 0.,
};

fn pose(mood: Mood, t: f32) -> Pose {
    let wave = |period: f32| (t / period * TAU).sin();
    match mood {
        Mood::Organizing => Pose {
            lift: 0.8 + 0.8 * wave(1.6),
            tilt: 3f32.to_radians() * wave(1.6),
            look: [2. * wave(3.2), -0.6 * (t / 3.2 * TAU).cos()],
            open: blink(t, 5.9),
            ripple: t / 3.6 * TAU,
            ..REST
        },
        Mood::Happy => {
            let (lift, squash) = hop((t % 1.5) / 1.5);
            Pose {
                lift,
                squash,
                ripple: t / 3.6 * TAU,
                ..REST
            }
        }
        Mood::Worried => Pose {
            sway: if (0.4..1.4).contains(&t) {
                0.8 * wave(1. / 12.)
            } else {
                0.
            },
            open: blink(t, 2.1),
            ripple: t / 1.4 * TAU,
            ..REST
        },
        Mood::Idle | Mood::Asleep => REST,
    }
}

/// One hop with squash and stretch; `progress` runs 0 to 1 and both ends are at rest.
fn hop(progress: f32) -> (f32, [f32; 2]) {
    /// (progress, lift, squash x, squash y)
    const KEYS: [(f32, f32, f32, f32); 7] = [
        (0., 0., 1., 1.),
        (0.12, 0., 1.08, 0.92),
        (0.3, 2., 0.94, 1.07),
        (0.5, 9., 1., 1.),
        (0.72, 0., 1.1, 0.9),
        (0.86, 0., 0.98, 1.02),
        (1., 0., 1., 1.),
    ];
    let index = KEYS
        .iter()
        .rposition(|key| key.0 <= progress)
        .unwrap_or(0)
        .min(KEYS.len() - 2);
    let (from, to) = (KEYS[index], KEYS[index + 1]);
    let linear = ((progress - from.0) / (to.0 - from.0)).clamp(0., 1.);
    let eased = linear * linear * (3. - 2. * linear);
    let mix = |start: f32, end: f32| start + (end - start) * eased;
    (mix(from.1, to.1), [mix(from.2, to.2), mix(from.3, to.3)])
}

/// Eyes close for a moment around `at` seconds.
fn blink(t: f32, at: f32) -> f32 {
    let distance = (t - at).abs() / 0.09;
    if distance < 1. {
        0.08 + 0.92 * distance
    } else {
        1.
    }
}

/// Maps design units through the pose onto the canvas, and paints in the mascot's ink.
struct Pen {
    origin: Point<Pixels>,
    scale: f32,
    pose: Pose,
    ink: Hsla,
    surface: Hsla,
}

impl Pen {
    fn new(bounds: Bounds<Pixels>, pose: Pose, ink: Hsla, surface: Hsla) -> Pen {
        Pen {
            origin: bounds.origin,
            scale: f32::from(bounds.size.height) / BOX_SIZE,
            pose,
            ink,
            surface,
        }
    }

    /// A design point, moved with the body.
    fn at(&self, [x, y]: [f32; 2]) -> Point<Pixels> {
        let Pose {
            squash: [sx, sy],
            tilt,
            ..
        } = self.pose;
        let (dx, dy) = ((x - FEET[0]) * sx, (y - FEET[1]) * sy);
        let (sin, cos) = tilt.sin_cos();
        let moved = [
            FEET[0] + dx * cos - dy * sin + self.pose.sway,
            FEET[1] + dx * sin + dy * cos - self.pose.lift,
        ];
        self.fixed(moved)
    }

    /// A design point that doesn't move with the body.
    fn fixed(&self, [x, y]: [f32; 2]) -> Point<Pixels> {
        point(
            self.origin.x + px((x - BOX_ORIGIN[0]) * self.scale),
            self.origin.y + px((y - BOX_ORIGIN[1]) * self.scale),
        )
    }

    /// The pressed dot: almost round, its edge slowly wobbling while it moves.
    fn body(&self, window: &mut Window) {
        const STEPS: u16 = 32;
        let ripple = self.pose.ripple;
        let edge: Vec<[f32; 2]> = (0..STEPS)
            .map(|i| {
                let a = f32::from(i) / f32::from(STEPS) * TAU;
                let r = RADIUS
                    * (1.
                        + 0.022 * (3. * a + 0.7 + ripple).sin()
                        + 0.014 * (5. * a + 2.1 - 1.3 * ripple).sin()
                        + 0.012 * (2. * a + 4. + 0.7 * ripple).sin());
                [CENTER[0] + r * a.cos(), CENTER[1] + r * a.sin() * 0.97]
            })
            .collect();
        // A Catmull-Rom curve through the edge points, as cubic Béziers.
        let mut path = PathBuilder::fill();
        path.move_to(self.at(edge[0]));
        let count = edge.len();
        for i in 0..count {
            let [p0, p1, p2, p3] = [count - 1, 0, 1, 2].map(|o| edge[(i + o) % count]);
            let a = [p1[0] + (p2[0] - p0[0]) / 6., p1[1] + (p2[1] - p0[1]) / 6.];
            let b = [p2[0] - (p3[0] - p1[0]) / 6., p2[1] - (p3[1] - p1[1]) / 6.];
            path.cubic_bezier_to(self.at(p2), self.at(a), self.at(b));
        }
        path.close();
        if let Ok(path) = path.build() {
            window.paint_path(path, self.ink);
        }
    }

    /// An eye: a dot squashed by the blink, cut from the body.
    fn eye(&self, center: [f32; 2], radius: f32, window: &mut Window) {
        let c = self.at(center);
        let (w, h) = (radius * self.scale, radius * self.scale * self.pose.open);
        let bounds = Bounds::new(
            point(c.x - px(w), c.y - px(h)),
            size(px(w * 2.), px(h * 2.)),
        );
        window.paint_quad(fill(bounds, self.surface).corner_radii(px(w.min(h))));
    }

    /// A quadratic pen line through design points, in the face's colour or the ink.
    fn line(
        &self,
        from: [f32; 2],
        ctrl: [f32; 2],
        to: [f32; 2],
        on_body: bool,
        window: &mut Window,
    ) {
        let place = |p| if on_body { self.at(p) } else { self.fixed(p) };
        let options = StrokeOptions::default()
            .with_line_width(LINE * self.scale)
            .with_line_cap(LineCap::Round)
            .with_line_join(LineJoin::Round);
        let mut path = PathBuilder::default().with_style(PathStyle::Stroke(options));
        path.move_to(place(from));
        path.curve_to(place(to), place(ctrl));
        if let Ok(path) = path.build() {
            window.paint_path(path, if on_body { self.surface } else { self.ink });
        }
    }

    /// A curve `width` wide centred on `at`; `bend` above zero sags, below zero arches.
    fn curve(&self, at: [f32; 2], width: f32, bend: f32, window: &mut Window) {
        let half = width / 2.;
        self.line(
            [at[0] - half, at[1]],
            [at[0], at[1] + width * bend],
            [at[0] + half, at[1]],
            true,
            window,
        );
    }
}

fn face(mood: Mood, pen: &Pen, window: &mut Window) {
    let shifted = |[x, y]: [f32; 2], dx: f32, dy: f32| [x + dx, y + dy];
    match mood {
        Mood::Idle => {
            for eye in EYES {
                pen.eye(eye, EYE_RADIUS, window);
            }
            pen.curve(MOUTH, 3.6, 0.45, window);
        }
        Mood::Organizing => {
            let [lx, ly] = pen.pose.look;
            for eye in EYES {
                pen.eye(shifted(eye, 1.2 + lx, -1.6 + ly), EYE_RADIUS, window);
            }
            pen.curve(shifted(MOUTH, 0., 0.6), 1.8, 0., window);
        }
        Mood::Happy => {
            for eye in EYES {
                pen.curve(shifted(eye, 0., 1.), 4.4, -0.62, window);
            }
            pen.curve(MOUTH, 5., 0.45, window);
        }
        Mood::Worried => {
            for eye in EYES {
                pen.eye(shifted(eye, 0., 0.6), EYE_RADIUS * 0.9, window);
            }
            let start = shifted(MOUTH, -2.4, 0.8);
            pen.curve(shifted(start, 1.2, 0.), 2.4, -0.4, window);
            pen.curve(shifted(start, 3.6, 0.), 2.4, 0.4, window);
        }
        Mood::Asleep => {
            for eye in EYES {
                pen.curve(shifted(eye, 0., 0.6), 4.2, 0.45, window);
            }
        }
    }
}

/// What floats around it: a drop of ink circling while it works, a "z" while it sleeps.
fn extras(mood: Mood, t: f32, pen: &Pen, window: &mut Window) {
    match mood {
        Mood::Organizing => {
            let a = t / 2.13 * TAU;
            let c = pen.fixed([CENTER[0] + 20.5 * a.sin(), CENTER[1] - 20.5 * a.cos()]);
            let r = px(2.1 * pen.scale);
            let bounds = Bounds::new(point(c.x - r, c.y - r), size(r * 2., r * 2.));
            window.paint_quad(fill(bounds, pen.ink).corner_radii(r));
        }
        Mood::Asleep => {
            pen.line([36., 6.5], [38.1, 6.5], [40.2, 6.5], false, window);
            pen.line([40.2, 6.5], [38.1, 8.8], [36., 11.1], false, window);
            pen.line([36., 11.1], [38.1, 11.1], [40.2, 11.1], false, window);
        }
        Mood::Idle | Mood::Happy | Mood::Worried => {}
    }
}
