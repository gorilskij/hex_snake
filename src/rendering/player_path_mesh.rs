use std::f32::consts::PI;
use std::iter;

use anyhow::Result;

use crate::app::game_context::GameContext;
use crate::app::stats::Stats;
use crate::apple::Apple;
use crate::basic::{Dir, Point};
use crate::rendering::shape::ShapePoints;
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::{SegmentType, Snake};
use crate::support::mesh::{build_circle, build_polygon, DrawMode, Mesh};
use crate::view::snakes::OtherSnakes;

/// The autopilot's path, as two meshes: the part to draw under the snakes,
/// and the part over the player's own eaten segments, which the path passes
/// through and which would otherwise hide it.
pub fn player_path_mesh(
    player_snake: &mut Snake,
    other_snakes: OtherSnakes,
    apples: &[Apple],
    gtx: &GameContext,
    stats: &mut Stats,
) -> Option<Result<(Mesh, Mesh)>> {
    let autopilot = player_snake.autopilot.as_mut()?;
    let knowledge = Knowledge::accurate(&player_snake.eat_mechanics);
    let path = autopilot.get_path(&player_snake.body, Some(&knowledge), &other_snakes, apples, gtx)?;

    // the head is where the path starts, so it stays over the path even when
    // eaten; only the body the path crosses further on is drawn under it
    let eaten: Vec<_> = player_snake
        .body
        .segments
        .iter()
        .skip(1)
        .filter(|segment| matches!(segment.segment_type, SegmentType::Eaten { .. }))
        .map(|segment| segment.pos)
        .collect();

    let mut under: Vec<Mesh> = vec![];
    let mut over: Vec<Mesh> = vec![];

    for (pos, next_pos) in path.iter().zip(path.iter().skip(1).map(Some).chain(iter::once(None))) {
        let dest = pos.to_cartesian(gtx.cell_dim) + gtx.cell_dim.center();

        // for the last point before a teleport, display a subtle hint about which direction
        // the snake should be going to teleport correctly
        let arrow = next_pos.and_then(|next_pos| {
            pos.single_step_dir_to(*next_pos, gtx.board_dim)
                .and_then(|dir| pos.explicit_wrapping_translate(dir, 1, gtx.board_dim).1.then_some(dir))
        });

        let radius = gtx.cell_dim.side / 2.5;
        let parts = if eaten.contains(pos) { &mut over } else { &mut under };

        parts.push(build_circle(DrawMode::fill(), dest, radius, crate::color::WHITE));
        stats.polygons += 1;

        if let Some(dir) = arrow {
            // the angle of the point of the arrow
            const THETA: f32 = PI * 3. / 8.;

            let cos = radius * (THETA / 2.).cos();
            let sin = radius * (THETA / 2.).sin();

            // pointing down
            let points = ShapePoints::from(vec![
                Point {
                    x: 0.,
                    y: radius / (THETA / 2.).sin(),
                },
                Point { x: cos, y: sin },
                Point { x: -cos, y: sin },
            ])
            .rotate_clockwise(Point::zero(), Dir::D.clockwise_angle_to(dir))
            .translate(dest);

            parts.push(build_polygon(DrawMode::fill(), &points, crate::color::WHITE));
            stats.polygons += 1;
        }
    }

    Some(Ok((Mesh::combine(under), Mesh::combine(over))))
}
