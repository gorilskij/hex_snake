use crate::app::game_context::GameContext;
use crate::app::portal::{Behavior, Portal};
use crate::app::stats::Stats;
use crate::basic::{CellDim, Dir, HexPoint, Point};
use crate::color::Color;
use crate::error::Result;
use crate::gfx::graphics::{build_line, Mesh};
use crate::rendering::shape::ShapePoints;

pub fn render_hexagon_edge(dir: Dir, CellDim { side, sin, cos }: CellDim) -> ShapePoints {
    use Dir::*;
    let points = match dir {
        // counterclockwise order
        D => vec![Point { x: cos + side, y: sin * 2. }, Point { x: cos, y: sin * 2. }],
        Dr => vec![
            Point { x: cos * 2. + side, y: sin },
            Point { x: cos + side, y: sin * 2. },
        ],
        Ur => vec![Point { x: cos + side, y: 0. }, Point { x: cos * 2. + side, y: sin }],
        U => vec![Point { x: cos, y: 0. }, Point { x: cos + side, y: 0. }],
        Ul => vec![Point { x: 0., y: sin }, Point { x: cos, y: 0. }],
        Dl => vec![Point { x: cos, y: sin * 2. }, Point { x: 0., y: sin }],
    };
    ShapePoints::from(points)
}

// TODO: make a build_full_edge function for when the half edges are the same
//       to avoid double drawing
fn build_half_edge(from: HexPoint, to: HexPoint, color: Color, gtx: &GameContext) -> Mesh {
    let dir = from
        .dir_to(to)
        .unwrap_or_else(|| panic!("invalid inputs: from {:?} to {:?}", from, to));
    let mut points = render_hexagon_edge(dir, gtx.cell_dim);

    let center = gtx.cell_dim.center();
    let from_center = from.to_cartesian(gtx.cell_dim) + center;
    let to_center = to.to_cartesian(gtx.cell_dim) + center;

    let location = from.to_cartesian(gtx.cell_dim) + from_center * 0.04 - to_center * 0.04;

    points = points.translate(location);

    build_line(&points, gtx.palette.border_thickness * 2.0, *color)
}

// TODO: make this part of palette
fn behavior_color(behavior: Behavior) -> Color {
    match behavior {
        Behavior::Die => Color::RED,
        Behavior::TeleportTo(_, _) => Color::from_rgb(50, 105, 168),
        Behavior::WrapAround => Color::WHITE,
        Behavior::PassThrough => Color::GREEN,
        Behavior::Nothing | Behavior::Unreachable => Color::TRANSPARENT,
    }
}

// TODO: update stats
pub fn portal_mesh(portals: &mut [Portal], gtx: &GameContext, _stats: &mut Stats) -> Result<Mesh> {
    let mut parts: Vec<Mesh> = vec![];

    for portal in portals {
        for edge in &portal.edges {
            let color_ab = behavior_color(edge.behavior_ab);
            parts.push(build_half_edge(edge.a, edge.b, color_ab, gtx));

            let color_ba = behavior_color(edge.behavior_ba);
            parts.push(build_half_edge(edge.b, edge.a, color_ba, gtx));
        }
    }

    Ok(Mesh::combine(parts))
}
