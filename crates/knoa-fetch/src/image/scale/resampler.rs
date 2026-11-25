//! A Catmull-Rom resampler fed one source row at a time. Each row is filtered across as it
//! arrives and kept in a window of the few rows the next output row needs, so shrinking
//! holds the output and that window, never a second copy of the image.

use std::ops::Range;

use image::{ColorType, DynamicImage};

/// Catmull-Rom reaches two source pixels either side, stretched by the shrink ratio so
/// every source pixel counts and fine detail doesn't alias.
const SUPPORT: f32 = 2.0;

pub struct Resampler {
    color: ColorType,
    channels: usize,
    width: u32,
    height: u32,
    columns: Columns,
    rows: Taps,
    /// The current output row's weights, worked out once for it.
    row_weights: Vec<f32>,
    /// Source rows filtered across, each `width` pixels long, in a ring.
    window: Vec<f32>,
    window_rows: usize,
    /// One output row being summed, before it is rounded into `output`.
    sum: Vec<f32>,
    received: usize,
    output: Vec<u8>,
}

impl Resampler {
    /// Shrinks `source` pixels of an 8-bit `color` to `target`, which is no larger.
    pub fn new(source: (u32, u32), target: (u32, u32), color: ColorType) -> Resampler {
        let channels = usize::from(color.channel_count());
        let rows = Taps::new(source.1, target.1);
        let row_len = target.0 as usize * channels;
        Resampler {
            color,
            channels,
            width: target.0,
            height: target.1,
            columns: Columns::new(Taps::new(source.0, target.0)),
            row_weights: Vec::with_capacity(rows.stride),
            window: vec![0.0; rows.stride * row_len],
            window_rows: rows.stride,
            rows,
            sum: vec![0.0; row_len],
            received: 0,
            output: Vec::with_capacity(target.1 as usize * row_len),
        }
    }

    /// The bytes a resampler holds: the output, the window of rows, one output row's sum
    /// and weights, and the columns' weights.
    pub fn bytes(source: (u32, u32), target: (u32, u32), channels: u8) -> u64 {
        let row_len = u64::from(target.0) * u64::from(channels);
        let window_rows = Taps::new(source.1, target.1).stride as u64;
        row_len * u64::from(target.1)
            + (window_rows + 1) * row_len * 4
            + window_rows * 4
            + Columns::bytes(&Taps::new(source.0, target.0))
    }

    /// Takes the next source row, and finishes every output row it completes.
    pub fn push_row(&mut self, row: &[u8]) {
        let row_len = self.sum.len();
        let slot = self.received % self.window_rows;
        let filtered = &mut self.window[slot * row_len..(slot + 1) * row_len];
        filter_across(row, &mut self.columns, self.channels, filtered);
        self.received += 1;
        while self.next_row_ready() {
            self.finish_row();
        }
    }

    /// The shrunk image; `None` when the source ran out of rows first.
    pub fn finish(self) -> Option<DynamicImage> {
        super::from_raw(self.width, self.height, self.color, self.output)
    }

    fn finished_rows(&self) -> usize {
        self.output.len() / self.sum.len()
    }

    fn next_row_ready(&self) -> bool {
        let row = self.finished_rows();
        row < self.height as usize && self.rows.span(row).end <= self.received
    }

    /// Sums the window's rows for the next output row. Its rows are all still in the ring:
    /// an output row is finished as soon as its last row arrives, and the ring holds as
    /// many rows as any output row reaches.
    fn finish_row(&mut self) {
        let row_len = self.sum.len();
        let row = self.finished_rows();
        let first = self.rows.span(row).start;
        self.rows.weights(row, &mut self.row_weights);
        self.sum.fill(0.0);
        for (offset, &weight) in self.row_weights.iter().enumerate() {
            let slot = (first + offset) % self.window_rows;
            let filtered = &self.window[slot * row_len..(slot + 1) * row_len];
            for (sum, value) in self.sum.iter_mut().zip(filtered) {
                *sum += weight * value;
            }
        }
        self.output
            .extend(self.sum.iter().map(|&value| to_byte(value)));
    }
}

/// `row` filtered across into `out`, one target pixel per weight set.
fn filter_across(row: &[u8], columns: &mut Columns, channels: usize, out: &mut [f32]) {
    for (x, pixel) in out.chunks_exact_mut(channels).enumerate() {
        let (first, weights) = columns.of(x);
        pixel.fill(0.0);
        let source = &row[first * channels..(first + weights.len()) * channels];
        for (&weight, sample) in weights.iter().zip(source.chunks_exact(channels)) {
            for (value, &byte) in pixel.iter_mut().zip(sample) {
                *value += weight * f32::from(byte);
            }
        }
    }
}

fn to_byte(value: f32) -> u8 {
    // Catmull-Rom's negative lobes overshoot past black and white at sharp edges.
    value.round().clamp(0.0, 255.0) as u8
}

/// Which source pixels each target pixel along one axis draws on, and with what weights,
/// summing to one. Worked out on demand, so an axis costs nothing however long it is.
struct Taps {
    source: usize,
    target: usize,
    ratio: f32,
    stretch: f32,
    /// The most source pixels any target pixel reaches.
    stride: usize,
}

impl Taps {
    fn new(source: u32, target: u32) -> Taps {
        let ratio = source as f32 / target as f32;
        let stretch = ratio.max(1.0);
        Taps {
            source: source as usize,
            target: target as usize,
            ratio,
            stretch,
            stride: (2.0 * SUPPORT * stretch).ceil() as usize + 1,
        }
    }

    fn center(&self, i: usize) -> f32 {
        (i as f32 + 0.5) * self.ratio
    }

    /// The source pixels target pixel `i` reaches, cut short at the edges.
    fn span(&self, i: usize) -> Range<usize> {
        let reach = SUPPORT * self.stretch;
        let center = self.center(i);
        let start = (center - reach).floor().max(0.0) as usize;
        let end = ((center + reach).ceil() as usize)
            .min(self.source)
            .min(start + self.stride);
        start..end
    }

    /// Target pixel `i`'s weights, one per pixel of its span, into `out`.
    fn weights(&self, i: usize, out: &mut Vec<f32>) {
        let center = self.center(i);
        out.clear();
        out.extend(
            self.span(i)
                .map(|k| catmull_rom((k as f32 + 0.5 - center) / self.stretch)),
        );
        let total: f32 = out.iter().sum();
        for weight in out.iter_mut() {
            *weight /= total;
        }
    }
}

/// Every source row needs every column's weights, so they are kept, unless an extreme
/// aspect ratio would make them large; then each is worked out again as it is needed.
struct Columns {
    taps: Taps,
    /// Each target pixel's weights, `stride` slots apart.
    kept: Option<Vec<f32>>,
    scratch: Vec<f32>,
}

/// Weights kept for the columns, at most: 4 MB.
const KEPT_WEIGHTS: usize = 1 << 20;

impl Columns {
    fn new(taps: Taps) -> Columns {
        let kept = Columns::keeps(&taps).then(|| {
            let mut weights = vec![0.0; taps.target * taps.stride];
            let mut one = Vec::with_capacity(taps.stride);
            for (i, slots) in weights.chunks_exact_mut(taps.stride).enumerate() {
                taps.weights(i, &mut one);
                slots[..one.len()].copy_from_slice(&one);
            }
            weights
        });
        Columns {
            scratch: Vec::with_capacity(taps.stride),
            taps,
            kept,
        }
    }

    fn bytes(taps: &Taps) -> u64 {
        let kept = if Columns::keeps(taps) {
            taps.target * taps.stride
        } else {
            0
        };
        // Weights are f32s; a target pixel's are worked out in a scratch row of them.
        ((kept + taps.stride) * 4) as u64
    }

    fn keeps(taps: &Taps) -> bool {
        taps.target * taps.stride <= KEPT_WEIGHTS
    }

    fn of(&mut self, x: usize) -> (usize, &[f32]) {
        let span = self.taps.span(x);
        if let Some(weights) = &self.kept {
            let start = x * self.taps.stride;
            (span.start, &weights[start..start + span.len()])
        } else {
            self.taps.weights(x, &mut self.scratch);
            (span.start, &self.scratch)
        }
    }
}

fn catmull_rom(x: f32) -> f32 {
    let x = x.abs();
    if x < 1.0 {
        (1.5 * x - 2.5) * x * x + 1.0
    } else if x < 2.0 {
        ((-0.5 * x + 2.5) * x - 4.0) * x + 2.0
    } else {
        0.0
    }
}
