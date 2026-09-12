//! Hints at the board's edges about what lies on the other side: every edge of
//! a border cell that wraps around the board shows a subtle gradient in its
//! cell, colored by what would happen to the player on crossing it (see
//! [`Outcome`]). Hints ease in, out, and between colors in real time.

use std::collections::HashMap;

use macroquad::color::Color;

use crate::app::screen::Environment;
use crate::app::snake_management::{outcome_at, Outcome};
use crate::basic::{Dir, HexDim, HexPoint, Point};
use crate::color::lerp;
use crate::rendering::shape::{Hexagon, Shape};
use crate::support::mesh::{build_colored_polygon, Mesh};
use crate::support::time::Instant;

/// Roughly how long (s) a hint takes to settle after what's behind it changes.
const FADE_TIME: f32 = 0.1;

/// How far a hint reaches into its cell, as a fraction of the way from the
/// edge to the cell's center.
const DEPTH: f32 = 0.5;

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
    /// its edge, and build the mesh.
    pub fn mesh(&mut self, env: &Environment, player_idx: usize) -> Mesh {
        let now = Instant::now();
        let elapsed = self.last_update.map_or(0., |last| (now - last).as_secs_f32());
        self.last_update = Some(now);
        // exponential approach, ~95% of the way there after FADE_TIME
        let step = 1. - (-3. * elapsed / FADE_TIME).exp();

        let palette = &env.gtx.palette;
        let targets: HashMap<(HexPoint, Dir), Color> = wrap_edges(env.gtx.board_dim)
            .filter_map(|(pos, dir, destination)| {
                let color = match outcome_at(env, player_idx, destination)? {
                    Outcome::Apple => palette.hint_apple_color,
                    Outcome::Pass => palette.hint_pass_color,
                    Outcome::Cut => palette.hint_cut_color,
                    Outcome::Crash => palette.hint_crash_color,
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
        // corners clockwise from the top-left: edge i (corner i to i + 1) faces Dir i
        let corners = Hexagon::raw_points(cell_dim);
        let center = Hexagon::center(cell_dim);
        Mesh::combine(self.colors.iter().map(|(&(pos, dir), &color)| {
            let origin = pos.to_cartesian(cell_dim);
            let start = corners[dir as usize];
            let end = corners[(dir as usize + 1) % 6];
            let inward = |corner: Point| corner + (center - corner) * DEPTH;
            let clear = Color { a: 0., ..color };
            build_colored_polygon(&[
                (origin + start, color),
                (origin + end, color),
                (origin + inward(end), clear),
                (origin + inward(start), clear),
            ])
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
