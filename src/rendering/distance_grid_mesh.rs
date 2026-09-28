use std::cmp::max;

use macroquad::color::Color;

use crate::app::distance_grid::{Distance, DistanceGrid};
use crate::app::game_context::GameContext;
use crate::color::lerp;
use crate::rendering::shape::{Hexagon, Shape};
use crate::support::mesh::{DrawMode, Mesh, build_polygon};

const ALPHA: f32 = 0.3;
const CLOSEST_COLOR: Color = Color::from_rgba(51, 204, 51, 255).with_alpha(ALPHA);
const MIDWAY_COLOR: Color = Color::from_rgba(255, 255, 0, 255).with_alpha(ALPHA);
const FARTHEST_COLOR: Color = Color::from_rgba(204, 0, 0, 255).with_alpha(ALPHA);

/// Each cell colored by its distance from the player's head, green to red.
/// `fade` runs from 0 (the previous map) to 1 (the current one); a cell only
/// in one of them fades in or out of transparent.
pub fn distance_grid_mesh(grid: &DistanceGrid, fade: f32, gtx: &GameContext) -> Mesh {
    // not actually max distance but a good estimate, anything
    // higher gets the same color
    let max_dist = max(gtx.board_dim.h, gtx.board_dim.v) as f32;
    let color_of = |dist: Distance| {
        let ratio = (dist / max_dist).min(1.);
        if ratio < 0.5 {
            lerp(CLOSEST_COLOR, MIDWAY_COLOR, ratio * 2.)
        } else {
            lerp(MIDWAY_COLOR, FARTHEST_COLOR, ratio * 2. - 1.)
        }
    };

    let parts = grid.cells().map(|(pos, before, now)| {
        let color = match (before.map(color_of), now.map(color_of)) {
            (Some(before), Some(now)) => lerp(before, now, fade),
            (Some(before), None) => lerp(before, before.with_alpha(0.), fade),
            (None, Some(now)) => lerp(now.with_alpha(0.), now, fade),
            (None, None) => unreachable!("every cell has a distance in one of the maps"),
        };
        let hexagon = Hexagon::new(gtx.cell_dim).translate(pos.to_cartesian(gtx.cell_dim));
        build_polygon(DrawMode::fill(), &hexagon, color)
    });
    Mesh::combine(parts)
}
