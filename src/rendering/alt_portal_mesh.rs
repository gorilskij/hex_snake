use anyhow::Result;
use macroquad::color::Color;

use crate::app::game_context::GameContext;
use crate::app::portal::{alt, Behavior};
use crate::app::stats::Stats;
use crate::basic::{CellDim, Dir, HexPoint, Point};
use crate::rendering::shape::ShapePoints;
use crate::support::mesh::{build_polygon, DrawMode, Mesh};

pub fn render_hexagon_edge(dir: Dir, cd @ CellDim { side, sin, cos }: CellDim) -> ShapePoints {
    use Dir::*;

    ShapePoints::from(vec![
        Point { x: cos, y: sin * 2. },
        Point { x: cos * 0.8, y: sin * 1.8 },
        Point { x: cos * 1.2 + side, y: sin * 1.8 },
        Point { x: cos + side, y: sin * 2. },
    ])
    .rotate_clockwise(cd.center(), D.clockwise_angle_to(dir))
}

// TODO: make a build_full_edge function for when the half edges are the same
//       to avoid double drawing
fn build_edge(from: HexPoint, to: HexPoint, color: Color, gtx: &GameContext) -> Mesh {
    let dir = from
        .dir_to(to)
        .unwrap_or_else(|| panic!("invalid inputs: from {:?} to {:?}", from, to));
    let mut points = render_hexagon_edge(dir, gtx.cell_dim);

    let location = to.to_cartesian(gtx.cell_dim);

    points = points.translate(location);

    build_polygon(DrawMode::fill(), &points, color)
}

// TODO: make this part of palette
fn behavior_color(behavior: Behavior) -> Color {
    match behavior {
        Behavior::Die => crate::color::RED,
        Behavior::TeleportTo(_, _) => Color::from_rgba(50, 105, 168, 255),
        Behavior::WrapAround => crate::color::WHITE,
        Behavior::PassThrough => crate::color::GREEN,
        Behavior::Nothing | Behavior::Unreachable => Color::new(0., 0., 0., 0.),
    }
}

// TODO: update stats
pub fn alt_portal_mesh(portals: &mut [alt::Portal], gtx: &GameContext, _stats: &mut Stats) -> Result<Mesh> {
    let mut parts: Vec<Mesh> = vec![];

    for portal in portals {
        for edge in &portal.edges {
            let color = behavior_color(edge.behavior);
            parts.push(build_edge(edge.from, edge.to, color, gtx));
        }
    }

    Ok(Mesh::combine(parts))
}
