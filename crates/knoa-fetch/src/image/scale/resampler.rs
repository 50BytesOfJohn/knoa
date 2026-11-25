//! A Catmull-Rom resampler fed one source row at a time. Each row is filtered across as it
//! arrives and kept in a window of the few rows the next output row needs, so shrinking
//! holds the output and that window, never a second copy of the image.

use image::{ColorType, DynamicImage};

/// Catmull-Rom reaches two source pixels either side, stretched by the shrink ratio so
/// every source pixel counts and fine detail doesn't alias.
const SUPPORT: f32 = 2.0;

pub struct Resampler {
    color: ColorType,
    channels: usize,
    width: u32,
    height: u32,
    columns: Taps,
    rows: Taps,
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
            columns: Taps::new(source.0, target.0),
            window: vec![0.0; rows.stride * row_len],
            window_rows: rows.stride,
            rows,
            sum: vec![0.0; row_len],
            received: 0,
            output: Vec::with_capacity(target.1 as usize * row_len),
        }
    }

    /// The bytes a resampler holds: the output, the window and the filter weights.
    pub fn bytes(source: (u32, u32), target: (u32, u32), channels: u8) -> u64 {
        let row_len = u64::from(target.0) * u64::from(channels);
        let window_rows = Taps::stride_for(source.1, target.1) as u64;
        // A weight is an f32; each target pixel also keeps its first source pixel and count.
        let weights =
            |from: u32, to: u32| u64::from(to) * (Taps::stride_for(from, to) as u64 * 4 + 16);
        row_len * u64::from(target.1)
            + (window_rows + 1) * row_len * 4
            + weights(source.0, target.0)
            + weights(source.1, target.1)
    }

    /// Takes the next source row, and finishes every output row it completes.
    pub fn push_row(&mut self, row: &[u8]) {
        let row_len = self.sum.len();
        let slot = self.received % self.window_rows;
        let filtered = &mut self.window[slot * row_len..(slot + 1) * row_len];
        filter_across(row, &self.columns, self.channels, filtered);
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
        row < self.height as usize && {
            let (first, weights) = self.rows.of(row);
            first + weights.len() <= self.received
        }
    }

    /// Sums the window's rows for the next output row. Its rows are all still in the ring:
    /// an output row is finished as soon as its last row arrives, and the ring holds as
    /// many rows as any output row reaches.
    fn finish_row(&mut self) {
        let row_len = self.sum.len();
        let (first, weights) = self.rows.of(self.finished_rows());
        self.sum.fill(0.0);
        for (offset, &weight) in weights.iter().enumerate() {
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
fn filter_across(row: &[u8], columns: &Taps, channels: usize, out: &mut [f32]) {
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

/// Each target pixel's first source pixel and weights along one axis, the weights summing
/// to one. Every pixel has `stride` slots; one near an edge uses fewer.
struct Taps {
    first: Vec<usize>,
    count: Vec<usize>,
    weights: Vec<f32>,
    stride: usize,
}

impl Taps {
    fn new(source: u32, target: u32) -> Taps {
        let ratio = source as f32 / target as f32;
        let stretch = ratio.max(1.0);
        let reach = SUPPORT * stretch;
        let stride = Taps::stride_for(source, target);
        let mut taps = Taps {
            first: Vec::with_capacity(target as usize),
            count: Vec::with_capacity(target as usize),
            weights: vec![0.0; target as usize * stride],
            stride,
        };
        for i in 0..target as usize {
            let center = (i as f32 + 0.5) * ratio;
            let first = (center - reach).floor().max(0.0) as usize;
            let end = ((center + reach).ceil() as usize)
                .min(source as usize)
                .min(first + stride);
            let slots = &mut taps.weights[i * stride..i * stride + (end - first)];
            for (k, weight) in (first..end).zip(slots.iter_mut()) {
                *weight = catmull_rom((k as f32 + 0.5 - center) / stretch);
            }
            let total: f32 = slots.iter().sum();
            for weight in slots.iter_mut() {
                *weight /= total;
            }
            taps.first.push(first);
            taps.count.push(end - first);
        }
        taps
    }

    fn stride_for(source: u32, target: u32) -> usize {
        let stretch = (source as f32 / target as f32).max(1.0);
        (2.0 * SUPPORT * stretch).ceil() as usize + 1
    }

    fn of(&self, i: usize) -> (usize, &[f32]) {
        let start = i * self.stride;
        (self.first[i], &self.weights[start..start + self.count[i]])
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
