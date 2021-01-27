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

pub fn generate_grid_mesh(
    ctx: &mut Context,
    dim: HexDim,
    palette: &GamePalette,
    cell_dim: CellDim,
) -> GameResult<Mesh> {
    let CellDim { side, sin, cos } = cell_dim;

    // two kinds of alternating vertical lines
    let mut vline_a = vec![];
    let mut vline_b = vec![];

    #[rustfmt::skip]
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
            #[rustfmt::skip]
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
                #[rustfmt::skip]
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
    let _ = ctx;
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

fn rotate_around_point(
    points: &[Point2<f32>],
    angle: f32,
    origin: Point2<f32>,
) -> Vec<Point2<f32>> {
    let sin = angle.sin();
    let cos = angle.cos();
    let Point2 { x: ox, y: oy } = origin;
    points
        .iter()
        .map(|Point2 { x, y }| {
            let tx = *x - ox;
            let ty = *y - oy;
            Point2 {
                x: tx * cos - ty * sin + ox,
                y: tx * sin + ty * cos + oy,
            }
        })
        .collect()
}

// polygon points to draw a snake segment
// from and to describe the relative position of the previous and next segments
pub fn get_points(
    dest: Point2<f32>,
    from: Option<Dir>,
    to: Option<Dir>,
    cell_dim: CellDim,
) -> Vec<Point2<f32>> {
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
            #[rustfmt::skip]
            FULL_HEXAGON = Some(vec![
                Point2 { x: cos, y: 0. },
                Point2 {
                    x: side + cos,
                    y: 0.,
                },
                Point2 {
                    x: side + 2. * cos,
                    y: sin,
                },
                Point2 {
                    x: side + cos,
                    y: 2. * sin,
                },
                Point2 {
                    x: cos,
                    y: 2. * sin,
                },
                Point2 { x: 0., y: sin },
            ]);

            CACHED_POINTS = Some(HashMap::new());

            let map = CACHED_POINTS.as_mut().unwrap();

            #[rustfmt::skip]
            let straight_segment = vec![
                Point2 { x: cos, y: 0. },
                Point2 { x: side + cos, y: 0. },
                Point2 { x: side + cos, y: 2. * sin },
                Point2 { x: cos, y: 2. * sin },
            ];
            #[rustfmt::skip]
            let end_segment = vec![
                Point2 { x: cos, y: sin },
                Point2 { x: side + cos, y: sin },
                Point2 { x: side + cos, y: 2. * sin },
                Point2 { x: cos, y: 2. * sin },
            ];
            #[rustfmt::skip]
            let blunt_turn_segment = vec![
                Point2 { x: cos, y: 0. },
                Point2 { x: side + cos, y: 0. },
                Point2 { x: side + 2. * cos, y: sin },
                Point2 { x: side + cos, y: 2. * sin },
            ];
            #[rustfmt::skip]
            let sharp_turn_segment = vec![
                Point2 { x: cos, y: 0. },
                Point2 { x: side + cos, y: 0. },
                Point2 { x: side + 2. * cos, y: sin },
                Point2 { x: cos, y: 2. * sin },
            ];

            let origin = cell_dim.center();

            for &dir in &[UL, U, UR] {
                let straight =
                    rotate_around_point(&straight_segment, dir.clockwise_angle_from_u(), origin);
                map.insert((Some(dir), Some(-dir)), straight);
            }

            for dir in Dir::iter() {
                let end = rotate_around_point(&end_segment, dir.clockwise_angle_from_u(), origin);
                map.insert((Some(-dir), None), end.clone());

                let blunt =
                    rotate_around_point(&blunt_turn_segment, dir.clockwise_angle_from_u(), origin);
                map.insert(
                    (Some(dir), Some(dir.next_clockwise().next_clockwise())),
                    blunt,
                );

                let sharp =
                    rotate_around_point(&sharp_turn_segment, dir.clockwise_angle_from_u(), origin);
                map.insert((Some(dir), Some(dir.next_clockwise())), sharp);
            }
        }

        let map = CACHED_POINTS.as_mut().unwrap();

        let mut points = match map.get(&(from, to)) {
            Some(ps) => ps,
            None => match map.get(&(to, from)) {
                Some(ps) => ps,
                None => FULL_HEXAGON.as_ref().unwrap(),
            },
        }
        .clone();

        for Point2 { x, y } in &mut points {
            *x += dest.x;
            *y += dest.y;
        }

        points
    }
}
