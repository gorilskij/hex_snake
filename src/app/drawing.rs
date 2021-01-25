use crate::app::{
    game::CellDim,
    hex::{Dir, HexDim},
    palette::GamePalette,
};
use ggez::{
    graphics::{DrawMode, Mesh, MeshBuilder},
    mint::Point2,
    Context, GameResult,
};
use num_integer::Integer;
use std::collections::HashMap;

#[rustfmt::skip]
pub fn generate_grid_mesh(ctx: &mut Context, dim: HexDim, palette: &GamePalette, cell_dim: CellDim) -> GameResult<Mesh> {
    let CellDim { side, sin, cos } = cell_dim;

    // two kinds of alternating vertical lines
    let mut vline_a = vec![];
    let mut vline_b = vec![];

    for dv in (0..=dim.v).map(|v| v as f32 * 2. * sin) {
        vline_a.push(Point2 { x: cos, y: dv });
        vline_a.push(Point2 { x: 0., y: dv + sin });
        vline_b.push(Point2 { x: cos + side, y: dv });
        vline_b.push(Point2 { x: 2. * cos + side, y: dv + sin });
    }

    let mut builder = MeshBuilder::new();

    let draw_mode = DrawMode::stroke(palette.grid_thickness);
    let color = palette.grid_color;
    for h in 0..(dim.h + 1) / 2 {
        if h == 0 {
            builder.polyline(draw_mode, &vline_a[..vline_a.len() - 1], color)?;
        } else {
            builder.polyline(draw_mode, &vline_a, color)?;
        }
        if dim.h.is_odd() && h == (dim.h + 1) / 2 - 1 {
            builder.polyline(draw_mode, &vline_b[..vline_b.len() - 1], color)?;
        } else {
            builder.polyline(draw_mode, &vline_b, color)?;
        }

        let dh = h as f32 * (2. * side + 2. * cos);

        for v in 0..=dim.v {
            let dv = v as f32 * 2. * sin;

            // line between a and b
            builder.line(
                &[
                    Point2 { x: cos + dh, y: dv },
                    Point2 { x: cos + side + dh, y: dv },
                ],
                palette.grid_thickness,
                color,
            )?;

            // line between b and a
            if !(dim.h.is_odd() && h == (dim.h + 1) / 2 - 1) {
                builder.line(
                    &[
                        Point2 { x: 2. * cos + side + dh, y: sin + dv },
                        Point2 { x: 2. * cos + 2. * side + dh, y: sin + dv },
                    ],
                    palette.grid_thickness,
                    color,
                )?;
            }
        }

        // shift the lines right by 2 cells
        let offset = 2. * (side + cos);
        vline_a.iter_mut().for_each(|a| a.x += offset);
        vline_b.iter_mut().for_each(|b| b.x += offset);
    }
    if dim.h.is_even() {
        builder.polyline(draw_mode, &vline_a[1..], color)?;
    }

    builder.build(ctx)
}

pub fn generate_border_mesh(ctx: &mut Context) -> GameResult<Mesh> {
    unimplemented!()
}

// positions stored in reverse order
struct ExceptPositionsIter<T, I: Iterator<Item = T>>(I, Vec<usize>, usize);

impl<T, I: Iterator<Item = T>> ExceptPositionsIter<T, I> {
    fn new(iter: I, positions: &[usize]) -> Self {
        let mut vec = positions.to_vec();
        vec.sort_by_key(|p| -(*p as isize));
        Self(iter, vec, 0)
    }
}

impl<T, I: Iterator<Item = T>> Iterator for ExceptPositionsIter<T, I> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let Self(iter, positions, pos) = self;

        while positions.last().map(|p| p == pos).unwrap_or(false) {
            let _ = iter.next()?;
            positions.pop();
            *pos += 1;
        }

        let next = iter.next()?;
        *pos += 1;
        Some(next)
    }
}

trait ExceptPositions {
    type Out;

    fn except_positions(self, positions: &[usize]) -> Self::Out;
}

impl<T: Copy> ExceptPositions for &[T] {
    type Out = Vec<T>;

    fn except_positions(self, positions: &[usize]) -> Self::Out {
        ExceptPositionsIter::new(self.iter().copied(), positions).collect()
    }
}

// polygon points to draw a snake segment
// from and to describe the relative position of the previous and next segments
#[rustfmt::skip]
pub fn get_points(dest: Point2<f32>, from: Option<Dir>, to: Option<Dir>, cell_dim: CellDim) -> Vec<Point2<f32>> {
    use Dir::*;

    let CellDim { side, sin, cos } = cell_dim;

    type FromTo = (Option<Dir>, Option<Dir>);
    static mut CACHED_SIDE: f32 = 0.;
    static mut FULL_HEXAGON: Option<Vec<Point2<f32>>> = None;
    static mut CACHED_POINTS: Option<HashMap<FromTo, Vec<Point2<f32>>>> = None;

    unsafe {
        if (side - CACHED_SIDE).abs() > f32::EPSILON {
            CACHED_SIDE = side;
            // starting from top-baseline/left, going clockwise
            FULL_HEXAGON = Some(vec![
                /* 0 */ Point2 { x: cos, y: 0. },
                /* 1 */ Point2 { x: side + cos, y: 0. },
                /* 2 */ Point2 { x: side + 2. * cos, y: sin },
                /* 3 */ Point2 { x: side + cos, y: 2. * sin },
                /* 4 */ Point2 { x: cos, y: 2. * sin },
                /* 5 */ Point2 { x: 0., y: sin },
            ]);
            CACHED_POINTS = Some(HashMap::new());

            // origin
            let Point2 { x: ox, y: oy } = cell_dim.center();
            let rotate = |points: &mut [Point2<f32>], angle: f32| {
                let sin = angle.sin();
                let cos = angle.cos();
                for Point2 { x, y } in points {
                    let tx = *x - ox;
                    let ty = *y - oy;
                    *x = tx * cos - ty * sin + ox;
                    *y = tx * sin + ty * cos + oy;
                }
            };

            let full_hexagon = FULL_HEXAGON.as_ref().unwrap();
            let map = CACHED_POINTS.as_mut().unwrap();

            // let straight_segment = vec![
            //     Point2 { x: cos, y: 0. },
            //     Point2 { x: side + cos, y: 0. },
            //     Point2 { x: side + cos, y: 2. * sin },
            //     Point2 { x: cos, y: 2. * sin },
            // ];
            // for dir in [Dir::U, Dir::UR, Dir::UL] {
            //     let mut clone = straight_segment.clone();
            //     rotate(&mut clone, dir.clockwise_angle_from_u());
            //     map.insert((Some(-dir), Some(dir)), clone);
            // }
            // straight lines
            map.insert((Some(D), Some(U)), full_hexagon.except_positions(&[2, 5]));
            map.insert((Some(DL), Some(UR)), full_hexagon.except_positions(&[0, 3]));
            map.insert((Some(DR), Some(UL)), full_hexagon.except_positions(&[1, 4]));

            // blunt turns
            map.insert((Some(D), Some(UL)), full_hexagon.except_positions(&[1, 2]));
            map.insert((Some(D), Some(UR)), full_hexagon.except_positions(&[0, 5]));
            map.insert((Some(U), Some(DL)), full_hexagon.except_positions(&[2, 3]));
            map.insert((Some(U), Some(DR)), full_hexagon.except_positions(&[4, 5]));
            map.insert((Some(UL), Some(UR)), full_hexagon.except_positions(&[3, 4]));
            map.insert((Some(DL), Some(DR)), full_hexagon.except_positions(&[0, 1]));

            // sharp turns
            map.insert((Some(U), Some(UL)), full_hexagon.except_positions(&[2, 4]));
            map.insert((Some(U), Some(UR)), full_hexagon.except_positions(&[3, 5]));
            map.insert((Some(D), Some(DL)), full_hexagon.except_positions(&[0, 2]));
            map.insert((Some(D), Some(DR)), full_hexagon.except_positions(&[1, 5]));
            map.insert((Some(UL), Some(DL)), full_hexagon.except_positions(&[1, 3]));
            map.insert((Some(UR), Some(DR)), full_hexagon.except_positions(&[0, 4]));


            // from top-left, clockwise
            // let a = Point2 { x: cos, y: 0. };
            // let b = Point2 { x: side + cos, y: 0. };
            // let c = Point2 { x: side + 2. * cos, y: sin };
            // let d = Point2 { x: side + cos, y: 2. * sin };
            // let e = Point2 { x: cos, y: 2. * sin };
            // let f = Point2 { x: 0., y: sin };
            //
            // // endpoint points (smaller hexagon, also clockwise)
            // // let g = Point2 { }
            // let g = Point2 { x: side + cos, y: sin };
            // let h = Point2 { x: cos, y: sin };

            // head
            // map.insert((Some(Dir::U), None), vec![a, b, g, h]);
            // map.insert((Some(Dir::D), None), vec![h, g, d, e]);
            // map.insert((Some(Dir::UL), None), vec![]);
            // map.insert((Some(Dir::UR), None), vec![
            //     Point2 { x: cos, y: 0. },
            //     Point2 { x: side + cos, y: 0. },
            //     Point2 { x: side + 2. * cos, y: sin },
            //     Point2 { x: side + cos, y: 2. * sin },
            //     Point2 { x: cos, y: 2. * sin },
            //     Point2 { x: 0., y: sin },
            // ]);
            // map.insert((Some(Dir::DL), None), vec![
            //     Point2 { x: cos, y: 0. },
            //     Point2 { x: side + cos, y: 0. },
            //     Point2 { x: side + 2. * cos, y: sin },
            //     Point2 { x: side + cos, y: 2. * sin },
            //     Point2 { x: cos, y: 2. * sin },
            //     Point2 { x: 0., y: sin },
            // ]);
            // map.insert((Some(Dir::DR), None), vec![
            //     Point2 { x: cos, y: 0. },
            //     Point2 { x: side + cos, y: 0. },
            //     Point2 { x: side + 2. * cos, y: sin },
            //     Point2 { x: side + cos, y: 2. * sin },
            //     Point2 { x: cos, y: 2. * sin },
            //     Point2 { x: 0., y: sin },
            // ]);
        }

        let map = CACHED_POINTS
            .as_mut()
            .unwrap();
        let maybe_points = match map.get(&(from, to)) {
            // if the segment is transitive, the direction doesn't matter
            None if from.is_some() && to.is_some() => map.get(&(to, from)),
            opt => opt,
        };
        let mut points = maybe_points
            .unwrap_or(FULL_HEXAGON.as_ref().unwrap())
            .clone();
        // println!("{:?}", points);

        for Point2 { x, y } in &mut points {
            *x += dest.x;
            *y += dest.y;
        }

        points
    }
    // let mut points = if from == Some(Dir::D) && to == Some(Dir::U) || from == Some(Dir::U) && to == Some(Dir::D) {
    //     vec![
    //         Point2 { x: cos, y: 0. },
    //         Point2 { x: side + cos, y: 0., },
    //         Point2 { x: side + cos, y: 2. * sin, },
    //         Point2 { x: cos, y: 2. * sin, },
    //     ]
    // } else if from == Some(Dir::DL) && to == Some(Dir::UR) || from == Some(Dir::UR) && to == Some(Dir::DL) {
    //     vec![
    //         Point2 { x: side + cos, y: 0., },
    //         Point2 { x: side + 2. * cos, y: sin, },
    //         Point2 { x: cos, y: 2. * sin, },
    //         Point2 { x: 0., y: sin },
    //     ]
    // } else if from == Some(Dir::DR) && to == Some(Dir::UL) || from == Some(Dir::UL) && to == Some(Dir::DR) {
    //     vec![
    //         Point2 { x: cos, y: 0. },
    //         Point2 { x: side + 2. * cos, y: sin, },
    //         Point2 { x: side + cos, y: 2. * sin, },
    //         Point2 { x: 0., y: sin },
    //     ]
    // } else if from == Some(Dir::D) && to == Some(Dir::UL) || from == Some(Dir::UL) && to == Some(Dir::D) {
    //     vec![
    //         Point2 { x: cos, y: 0. },
    //         Point2 { x: side + cos, y: 2. * sin, },
    //         Point2 { x: cos, y: 2. * sin, },
    //         Point2 { x: 0., y: sin },
    //     ]
    // } else if from == Some(Dir::D) && to == Some(Dir::UR) || from == Some(Dir::UR) && to == Some(Dir::D) {
    //     vec![
    //         Point2 { x: side + cos, y: 0., },
    //         Point2 { x: side + 2. * cos, y: sin, },
    //         Point2 { x: side + cos, y: 2. * sin, },
    //         Point2 { x: cos, y: 2. * sin, },
    //     ]
    // } else if from == Some(Dir::U) && to == Some(Dir::DL) || from == Some(Dir::DL) && to == Some(Dir::U) {
    //     vec![
    //         Point2 { x: cos, y: 0. },
    //         Point2 { x: side + cos, y: 0., },
    //         Point2 { x: cos, y: 2. * sin, },
    //         Point2 { x: 0., y: sin },
    //     ]
    // } else if from == Some(Dir::U) && to == Some(Dir::DR) || from == Some(Dir::DR) && to == Some(Dir::U) {
    //     vec![
    //         Point2 { x: cos, y: 0. },
    //         Point2 { x: side + cos, y: 0., },
    //         Point2 { x: side + 2. * cos, y: sin, },
    //         Point2 { x: side + cos, y: 2. * sin, },
    //     ]
    // } else if from == Some(Dir::UL) && to == Some(Dir::UR) || from == Some(Dir::UR) && to == Some(Dir::UL) {
    //     vec![
    //         Point2 { x: cos, y: 0. },
    //         Point2 { x: side + cos, y: 0., },
    //         Point2 { x: side + 2. * cos, y: sin, },
    //         Point2 { x: 0., y: sin },
    //     ]
    // } else if from == Some(Dir::DL) && to == Some(Dir::DR) || from == Some(Dir::DR) && to == Some(Dir::DL) {
    //     vec![
    //         Point2 { x: side + 2. * cos, y: sin, },
    //         Point2 { x: side + cos, y: 2. * sin, },
    //         Point2 { x: cos, y: 2. * sin, },
    //         Point2 { x: 0., y: sin },
    //     ]
    // } else if from == Some(Dir::U) && to == Some(Dir::UL) || from == Some(Dir::UL) && to == Some(Dir::U) {
    //     vec![
    //         Point2 { x: cos, y: 0. },
    //         Point2 { x: side + cos, y: 0., },
    //         Point2 { x: side + cos, y: 2. * sin, },
    //         Point2 { x: 0., y: sin },
    //     ]
    // } else if from == Some(Dir::U) && to == Some(Dir::UR) || from == Some(Dir::UR) && to == Some(Dir::U) {
    //     vec![
    //         Point2 { x: cos, y: 0. },
    //         Point2 { x: side + cos, y: 0., },
    //         Point2 { x: side + 2. * cos, y: sin, },
    //         Point2 { x: cos, y: 2. * sin, },
    //     ]
    // } else if from == Some(Dir::D) && to == Some(Dir::DL) || from == Some(Dir::DL) && to == Some(Dir::D) {
    //     vec![
    //         Point2 { x: side + cos, y: 0., },
    //         Point2 { x: side + cos, y: 2. * sin, },
    //         Point2 { x: cos, y: 2. * sin, },
    //         Point2 { x: 0., y: sin },
    //     ]
    // } else if from == Some(Dir::D) && to == Some(Dir::DR) || from == Some(Dir::DR) && to == Some(Dir::D) {
    //     vec![
    //         Point2 { x: cos, y: 0. },
    //         Point2 { x: side + 2. * cos, y: sin, },
    //         Point2 { x: side + cos, y: 2. * sin, },
    //         Point2 { x: cos, y: 2. * sin, },
    //     ]
    // } else if from == Some(Dir::UL) && to == Some(Dir::DL) || from == Some(Dir::DL) && to == Some(Dir::UL) {
    //     vec![
    //         Point2 { x: cos, y: 0. },
    //         Point2 { x: side + 2. * cos, y: sin, },
    //         Point2 { x: cos, y: 2. * sin, },
    //         Point2 { x: 0., y: sin },
    //     ]
    // } else if from == Some(Dir::UR) && to == Some(Dir::DR) || from == Some(Dir::DR) && to == Some(Dir::UR) {
    //     vec![
    //         Point2 { x: side + cos, y: 0., },
    //         Point2 { x: side + 2. * cos, y: sin, },
    //         Point2 { x: side + cos, y: 2. * sin, },
    //         Point2 { x: 0., y: sin },
    //     ]
    // } else {
    //     vec![
    //         Point2 { x: cos, y: 0. },
    //         Point2 { x: side + cos, y: 0., },
    //         Point2 { x: side + 2. * cos, y: sin, },
    //         Point2 { x: side + cos, y: 2. * sin, },
    //         Point2 { x: cos, y: 2. * sin, },
    //         Point2 { x: 0., y: sin },
    //     ]
    // };
    //
    // for Point2 { x, y } in &mut points {
    //     *x += dest.x;
    //     *y += dest.y;
    // }
    //
    // points
}
