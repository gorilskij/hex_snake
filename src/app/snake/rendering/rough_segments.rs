use crate::{
    app::snake::rendering::{descriptions::SegmentFraction, point_factory::SegmentRenderer},
    basic::{CellDim, Point},
};

pub struct RoughSegments;

impl SegmentRenderer for RoughSegments {
    fn render_default_straight(cell_dim: CellDim, fraction: SegmentFraction) -> Vec<Point> {
        let CellDim { side, sin, cos } = cell_dim;
        let SegmentFraction { start, end } = fraction;

        // top-left, top-right, bottom-right, bottom-left
        vec![
            Point { x: cos, y: end * 2. * sin },
            Point { x: cos + side, y: end * 2. * sin },
            Point { x: cos + side, y: start * 2. * sin },
            Point { x: cos, y: start * 2. * sin },
        ]
    }

    fn render_default_blunt(cell_dim: CellDim, fraction: SegmentFraction) -> Vec<Point> {
        let CellDim { side, sin, cos } = cell_dim;
        let SegmentFraction { start, end } = fraction;

        // The segment is defined by two parallel lines, A and B
        let a_start = Point { x: cos, y: 0. };
        let a_end = Point { x: cos + side, y: 2. * sin };
        let b_start = Point { x: cos + side, y: 0. };
        let b_end = Point { x: 2. * cos + side, y: sin };

        vec![
            (1. - start) * a_start + start * a_end,
            (1. - start) * b_start + start * b_end,
            (1. - end) * b_start + end * b_end,
            (1. - end) * a_start + end * a_end,
        ]
    }

    fn render_default_sharp(cell_dim: CellDim, fraction: SegmentFraction) -> Vec<Point> {
        let CellDim { side, sin, cos } = cell_dim;
        let SegmentFraction { start, end } = fraction;

        // The point around which the segment 'rotates'
        let pivot = Point { x: cos + side, y: 0. };

        // The three points it touches as it goes around
        // the first half of the animation is from a to b,
        // the second half is from b to c
        let a = Point { x: cos, y: 0. };
        let b = cell_dim.center();
        let c = Point { x: 2. * cos + side, y: sin };

        let mut points = Vec::with_capacity(4);

        points.push(pivot);
        if end >= 0.5 {
            let end = (end - 0.5) / 0.5;
            points.push((1. - end) * b + end * c);
            if start < 0.5 {
                let start = start / 0.5;
                points.push(b);
                points.push((1. - start) * a + start * b);
            } else {
                let start = (start - 0.5) / 0.5;
                points.push((1. - start) * b + start * c);
            }
        } else {
            let end = end / 0.5;
            points.push((1. - end) * a + end * b);
            // assume start <= end, so start < 0.5
            let start = start / 0.5;
            points.push((1. - start) * a + start * b);
        }

        println!("shaaaaaaaaaaaaaarp");

        // if end < 1. {
        //     if end >= 0.5 {
        //         let f = (end - 0.5) / 0.5;
        //         points.push(f * a + (1. - f) * b);
        //         points.push(b);
        //     } else {
        //         let f = end / 0.5;
        //         points.push(f * b + (1. - f) * c);
        //     }
        //     points.push(c);
        //     points.push(pivot);
        // } else {
        //     if start < 0.5 {
        //         let f = (start - 0.5) / 0.5;
        //         points.push(b);
        //         points.push(f * b + (1. - f) * c);
        //     } else {
        //         let f = start / 0.5;
        //         points.push(f * a + (1. - f) * b);
        //     }
        //     points.push(pivot);
        //     points.push(a);
        // }

        points
    }
}
