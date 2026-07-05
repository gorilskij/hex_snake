use std::f32::consts::PI;
use std::iter;

use crate::app::game_context::GameContext;
use crate::app::stats::Stats;
use crate::apple::Apple;
use crate::basic::{Dir, Point};
use anyhow::Result;
use crate::gfx::graphics::{build_circle, build_polygon, Color, DrawMode, Mesh};
use crate::rendering::shape::ShapePoints;
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Snake;
use crate::view::snakes::OtherSnakes;

pub fn player_path_mesh(
    player_snake: &mut Snake,
    other_snakes: OtherSnakes,
    apples: &[Apple],
    gtx: &GameContext,
    stats: &mut Stats,
) -> Option<Result<Mesh>> {
    let autopilot = player_snake.autopilot.as_mut()?;
    // TODO: this conversion is too expensive
    let knowledge = Knowledge::accurate(&player_snake.eat_mechanics);
    let path = autopilot.get_path(&player_snake.body, Some(&knowledge), &other_snakes, apples, gtx)?;

    let mut parts: Vec<Mesh> = vec![];

    for (pos, next_pos) in path.iter().zip(path.iter().skip(1).map(Some).chain(iter::once(None))) {
        let dest = pos.to_cartesian(gtx.cell_dim) + gtx.cell_dim.center();

        // for the last point before a teleport, display a subtle hint about which direction
        // the snake should be going to teleport correctly
        let arrow = next_pos.and_then(|next_pos| {
            pos.single_step_dir_to(*next_pos, gtx.board_dim)
                .and_then(|dir| pos.explicit_wrapping_translate(dir, 1, gtx.board_dim).1.then_some(dir))
        });

        let radius = gtx.cell_dim.side / 2.5;

        parts.push(build_circle(DrawMode::fill(), dest, radius, Color::WHITE));
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

            parts.push(build_polygon(DrawMode::fill(), &points, Color::WHITE));
            stats.polygons += 1;
        }
    }

    Some(Ok(Mesh::combine(parts)))
}
