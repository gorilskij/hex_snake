use ggez::graphics::{DrawMode, Mesh, MeshBuilder};
use crate::app::app_error::AppResult;
use crate::app::game_context::GameContext;
use crate::app::portal::Behavior;
use crate::basic::{CellDim, Dir, HexPoint, Point};
use crate::basic::transformations::translate;
use crate::color::Color;
use super::Portal;

pub fn render_hexagon_edge(dir: Dir, CellDim { side, sin, cos }: CellDim) -> Vec<Point> {
    use Dir::*;
    // TODO: translate the edges into the "from" hexagon
    //       so they don't overlap with the "to" hexagon
    match dir {
        // counterclockwise order
        D  => vec![
            Point { x: cos, y: 0. },
            Point { x: cos + side, y: 0. },
        ],
        Dr => {
            println!("triggered");
            vec![
                Point { x: cos + side, y: 0. },
                Point { x: cos * 2. + side, y: sin },
            ]
        },
        Ur => vec![
            Point { x: cos * 2. + side, y: sin },
            Point { x: cos + side, y: sin * 2. },
        ],
        U  => vec![
            Point { x: cos + side, y: sin * 2. },
            Point { x: cos, y: sin * 2. },
        ],
        Ul => vec![
            Point { x: cos, y: sin * 2. },
            Point { x: 0., y: sin },
        ],
        Dl => vec![
            Point { x: 0., y: sin },
            Point { x: cos, y: 0. },
        ],
    }
}

// TODO: make a build_full_edge function for when the half edges are the same
//       to avoid double drawing
fn build_half_edge(
    from: HexPoint,
    to: HexPoint,
    color: Color,
    gtx: &GameContext,
    builder: &mut MeshBuilder
) -> AppResult {
    println!("from {:?}", from);
    println!("to {:?}", to);
    let dir = from.dir_to(to)
        .unwrap_or_else(|| panic!("invalid inputs: from {:?} to {:?}", from, to));
    println!("dir {:?}", dir);
    let mut points = render_hexagon_edge(dir, gtx.cell_dim);
    translate(&mut points, to.to_cartesian(gtx.cell_dim));

    builder.line(&points, gtx.palette.border_thickness / 2.0, *color)?;
    Ok(())
}

// TODO: make this part of palette
fn color_of_behavior(behavior: Behavior) -> Color {
    match behavior {
        Behavior::Die => Color::RED,
        Behavior::Teleport => Color::WHITE,
        Behavior::PassThrough => Color::GREEN,
    }
}

impl Portal {
    pub fn build(&self, gtx: &GameContext, builder: &mut MeshBuilder) -> AppResult {
        for edge in &self.edges {
            let color_ab = color_of_behavior(edge.behavior_ab);
            build_half_edge(
                edge.a,
                edge.b,
                color_ab,
                gtx,
                builder,
            )?;

            let color_ba = color_of_behavior(edge.behavior_ba);
            build_half_edge(
                edge.b,
                edge.a,
                color_ba,
                gtx,
                builder,
            )?;
        }
        Ok(())
    }
}
