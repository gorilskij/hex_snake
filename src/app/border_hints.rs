//! Hints at the board's edges about what lies on the other side: every side of
//! a border cell that wraps around the board shows what would happen to the
//! player on crossing it (see [`Outcome`]), in one of two styles — recoloring
//! its stretch of the border ([`HintStyle::Border`]), or a gradient fading from
//! it into the cell ([`HintStyle::Gradient`]). Hints ease in, out, and between
//! colors in real time.

use std::collections::HashMap;

use macroquad::color::Color;

use crate::app::prefs::HintStyle;
use crate::app::screen::Environment;
use crate::app::snake_management::{outcome_at, Outcome};
use crate::basic::{Dir, HexDim, HexPoint, Point};
use crate::color::lerp;
use crate::rendering::shape::{Hexagon, Shape};
use crate::support::mesh::{build_colored_polygon, build_polygon, DrawMode, Mesh};
use crate::support::time::Instant;

/// Roughly how long (s) a hint takes to settle after what's behind it changes.
const FADE_TIME: f32 = 0.1;

/// How far a gradient hint reaches into its cell, as a fraction of the way from
/// the edge to the cell's center.
const GRADIENT_DEPTH: f32 = 0.5;

pub struct BorderHints {
    /// The displayed color of every edge whose hint is (still) visible.
    colors: HashMap<(HexPoint, Dir), Color>,
    last_update: Option<Instant>,
}

impl BorderHints {
    pub fn new() -> Self {
        Self {
            colors: HashMap::new(),
            last_update: None,
        }
    }

    /// Drop all hints at once (e.g. when the board changes shape).
    pub fn clear(&mut self) {
        self.colors.clear();
    }

    /// Ease every hint towards what the player would currently run into across
    /// its edge, and build the mesh in the given style.
    pub fn mesh(&mut self, env: &Environment, player_idx: usize, style: HintStyle) -> Mesh {
        let palette = &env.gtx.palette;
        let hint_colors = match style {
            HintStyle::Border => palette.border_hint_colors,
            HintStyle::Gradient => palette.gradient_hint_colors,
            HintStyle::None => {
                self.clear();
                return Mesh::empty();
            }
        };

        let now = Instant::now();
        let elapsed = self.last_update.map_or(0., |last| (now - last).as_secs_f32());
        self.last_update = Some(now);
        // exponential approach, ~95% of the way there after FADE_TIME
        let step = 1. - (-3. * elapsed / FADE_TIME).exp();

        let targets: HashMap<(HexPoint, Dir), Color> = wrap_edges(env.gtx.board_dim)
            .filter_map(|(pos, dir, destination)| {
                let color = match outcome_at(env, player_idx, destination)? {
                    Outcome::Apple => hint_colors.apple,
                    Outcome::Pass => hint_colors.pass,
                    Outcome::Cut => hint_colors.cut,
                    Outcome::Crash => hint_colors.crash,
                };
                Some(((pos, dir), color))
            })
            .collect();

        // a hint appears and disappears as a fade of its own color's opacity
        for (edge, target) in &targets {
            self.colors.entry(*edge).or_insert(Color { a: 0., ..*target });
        }
        self.colors.retain(|edge, color| {
            let target = targets.get(edge);
            *color = lerp(*color, target.copied().unwrap_or(Color { a: 0., ..*color }), step);
            target.is_some() || color.a > 1e-3
        });

        let cell_dim = env.gtx.cell_dim;
        let board_dim = env.gtx.board_dim;
        let half_width = palette.border_thickness / 2.;
        // corners clockwise from the top-left: side i (corner i to i + 1) faces Dir i
        let corners = Hexagon::raw_points(cell_dim);
        let center = Hexagon::center(cell_dim);

        Mesh::combine(self.colors.iter().map(|(&(pos, dir), &color)| {
            let origin = pos.to_cartesian(cell_dim);
            let corner = |i: usize| origin + corners[i % 6];
            let side = dir as usize;
            let (start, end) = (corner(side), corner(side + 1));

            match style {
                HintStyle::Border => {
                    // A band the border's width, centered on the side like the
                    // border itself, with each end cut along the line halving the
                    // corner the border turns at, so it meets the next stretch of
                    // border (hinted or not) exactly. If the cell's other side at
                    // that corner is on the border too, the border turns around
                    // this cell and the cut points at its center; otherwise it
                    // turns onto the neighboring cell and the cut runs along the
                    // side between the two.
                    let on_border = |side: usize| !board_dim.contains(pos.translate(Dir::from(side as u8), 1));
                    let cut = |vertex: Point, other_side: usize, other_corner: Point| {
                        if on_border(other_side) {
                            origin + center - vertex
                        } else {
                            other_corner - vertex
                        }
                    };

                    let along = (end - start) / (end - start).magnitude();
                    let normal = Point { x: -along.y, y: along.x };
                    // the points on a cut (through `vertex`) half a width to either side
                    let cut_ends = |vertex: Point, cut: Point| {
                        let reach = cut * (half_width / (cut.x * normal.x + cut.y * normal.y));
                        (vertex + reach, vertex - reach)
                    };

                    let (start_left, start_right) = cut_ends(start, cut(start, side + 5, corner(side + 5)));
                    let (end_left, end_right) = cut_ends(end, cut(end, side + 1, corner(side + 2)));
                    build_polygon(DrawMode::Fill, &[start_left, end_left, end_right, start_right], color)
                }
                HintStyle::Gradient => {
                    let inward = |corner: Point| corner + (origin + center - corner) * GRADIENT_DEPTH;
                    let clear = Color { a: 0., ..color };
                    build_colored_polygon(&[
                        (start, color),
                        (end, color),
                        (inward(end), clear),
                        (inward(start), clear),
                    ])
                }
                HintStyle::None => unreachable!("returned above"),
            }
        }))
    }
}

/// Every edge of a cell that wraps around the board, as `(cell, direction,
/// the cell it wraps to)`.
fn wrap_edges(board_dim: HexDim) -> impl Iterator<Item = (HexPoint, Dir, HexPoint)> {
    // a single step changes h and v by at most 1, so only the outermost rows and
    // columns can leave the board
    let on_border = move |pos: &HexPoint| {
        pos.h == 0 || pos.v == 0 || pos.h == board_dim.h - 1 || pos.v == board_dim.v - 1
    };
    (0..board_dim.h)
        .flat_map(move |h| (0..board_dim.v).map(move |v| HexPoint { h, v }))
        .filter(on_border)
        .flat_map(move |pos| Dir::iter().filter_map(move |dir| Some((pos, dir, pos.wrap_destination(dir, board_dim)?))))
}
