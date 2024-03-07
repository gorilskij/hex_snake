use ggez::graphics::{DrawMode, Mesh, MeshBuilder};
use ggez::Context;

use crate::app::fps_control::FpsContext;
use crate::app::game_context::GameContext;
use crate::app::portal::{Behavior, Portal};
use crate::app::stats::Stats;
use crate::basic::{CellDim, Dir, HexPoint, Point};
use crate::color::Color;
use crate::error::{ErrorConversion, Result};
use crate::rendering::shape::ShapePoints;

pub fn render_hexagon_edge(dir: Dir, CellDim { side, sin, cos }: CellDim) -> ShapePoints {
    use Dir::*;
    // TODO: translate the edges into the "from" hexagon
    //       so they don't overlap with the "to" hexagon
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
fn build_half_edge(from: HexPoint, to: HexPoint, color: Color, gtx: &GameContext, builder: &mut MeshBuilder) -> Result {
    println!("from {:?}", from);
    println!("to {:?}", to);
    let dir = from
        .dir_to(to)
        .unwrap_or_else(|| panic!("invalid inputs: from {:?} to {:?}", from, to));
    println!("dir: {:?}, color: {:?}", dir, color);
    let mut points = render_hexagon_edge(dir, gtx.cell_dim);

    let center = gtx.cell_dim.center();
    let from_center = from.to_cartesian(gtx.cell_dim) + center;
    let to_center = to.to_cartesian(gtx.cell_dim) + center;

    let location = from.to_cartesian(gtx.cell_dim) + from_center * 0.04 - to_center * 0.04;

    points = points.translate(location);

    builder.line(&points, gtx.palette.border_thickness * 2.0, *color)?;
    Ok(())
}

// TODO: make this part of palette
fn color_of_behavior(behavior: Behavior) -> Color {
    match behavior {
        Behavior::Die => Color::RED,
        Behavior::Teleport => Color::from_rgb(50, 105, 168),
        Behavior::WrapAround => Color::WHITE,
        Behavior::PassThrough => Color::GREEN,
    }
}

pub fn portal_mesh(
    portals: &[Portal],
    gtx: &GameContext,
    ftx: &FpsContext,
    ctx: &Context,
    stats: &mut Stats,
) -> Result<Mesh> {
    let builder = &mut MeshBuilder::new();

    let res: Result<_> = try {
        for portal in portals {
            for edge in &portal.edges {
                let color_ab = color_of_behavior(edge.behavior_ab);
                build_half_edge(edge.a, edge.b, color_ab, gtx, builder)?;

                let color_ba = color_of_behavior(edge.behavior_ba);
                build_half_edge(edge.b, edge.a, color_ba, gtx, builder)?;
            }
        }

        Mesh::from_data(ctx, builder.build())
    };

    res.with_trace_step("apple_mesh")
}
