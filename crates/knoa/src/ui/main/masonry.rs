//! Masonry layout (ADR 0007): every card's position computed from its known height, so the
//! view only renders the cards that intersect the viewport.

use super::groups::Group;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Default)]
pub struct Layout {
    pub cards: Vec<Placed>,
    /// The y of each group's header, in group order: date labels and the date rail.
    pub headers: Vec<f32>,
    pub height: f32,
}

pub struct Spec {
    pub width: f32,
    pub columns: usize,
    pub gap: f32,
    pub header_height: f32,
    /// Space above the first group (for the Up next strip).
    pub top: f32,
}

impl Layout {
    /// Places each group's cards into the currently shortest column, below the group's
    /// header. `height_of(index, column_width)` gives a card's height.
    pub fn masonry(groups: &[Group], spec: &Spec, height_of: impl Fn(usize, f32) -> f32) -> Layout {
        let columns = spec.columns.max(1);
        let column_width =
            ((spec.width - spec.gap * (columns - 1) as f32) / columns as f32).max(1.);
        let mut layout = Layout::default();
        let mut y = spec.top;
        for group in groups {
            layout.headers.push(y);
            let start = y + spec.header_height;
            let mut bottoms = vec![start; columns];
            for index in group.start..group.start + group.len {
                let (column, top) = bottoms
                    .iter()
                    .copied()
                    .enumerate()
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .unwrap_or((0, start));
                let height = height_of(index, column_width);
                let x = column as f32 * (column_width + spec.gap);
                layout.cards.push(Placed {
                    index,
                    x,
                    y: top,
                    width: column_width,
                    height,
                });
                bottoms[column] = top + height + spec.gap;
            }
            y = bottoms.into_iter().fold(start, f32::max) + spec.gap;
        }
        layout.height = y;
        layout
    }

    /// Cards that overlap the band from `top` to `bottom`.
    pub fn visible(&self, top: f32, bottom: f32) -> impl Iterator<Item = &Placed> {
        self.cards
            .iter()
            .filter(move |c| c.y + c.height >= top && c.y <= bottom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cards_go_to_the_shortest_column_and_groups_stack() {
        let groups = [
            Group {
                period: None,
                label: "a".into(),
                start: 0,
                len: 3,
            },
            Group {
                period: None,
                label: "b".into(),
                start: 3,
                len: 1,
            },
        ];
        let spec = Spec {
            width: 210.,
            columns: 2,
            gap: 10.,
            header_height: 20.,
            top: 0.,
        };
        let heights = [100., 40., 30.];
        let layout = Layout::masonry(&groups, &spec, |i, _| {
            heights.get(i).copied().unwrap_or(50.)
        });

        let at: Vec<(f32, f32)> = layout.cards.iter().map(|c| (c.x, c.y)).collect();
        // The third card goes under the short second one, not the tall first one.
        assert_eq!(at, [(0., 20.), (110., 20.), (110., 70.), (0., 160.)]);
        assert_eq!(layout.headers, [0., 140.]);
    }
}
