use std::f32::consts::PI;
use std::iter;

use ggez::graphics::{Color, DrawMode, Mesh, MeshBuilder};
use ggez::Context;

use crate::app::game_context::GameContext;
use crate::app::stats::Stats;
use crate::apple::Apple;
use crate::basic::{Dir, Point};
use crate::error::{ErrorConversion, Result};
use crate::rendering::shape::ShapePoints;
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Snake;
use crate::view::snakes::OtherSnakes;

pub fn player_path_mesh(
    player_snake: &mut Snake,
    other_snakes: OtherSnakes,
    apples: &[Apple],
    ctx: &Context,
    gtx: &GameContext,
    stats: &mut Stats,
) -> Option<Result<Mesh>> {
    let autopilot = player_snake.autopilot.as_mut()?;
    // TODO: this conversion is too expensive
    let knowledge = Knowledge::accurate(&player_snake.eat_mechanics);
    let path = autopilot.get_path(&player_snake.body, Some(&knowledge), &other_snakes, apples, gtx)?;

    let mut builder = MeshBuilder::new();

    let res = path
        .iter()
        .zip(path.iter().skip(1).map(Some).chain(iter::once(None)))
        .try_for_each(|(pos, next_pos)| {
            let dest = pos.to_cartesian(gtx.cell_dim) + gtx.cell_dim.center();

            // for the last point before a teleport, display a subtle hint about which direction
            // the snake should be going to teleport correctly
            let arrow = next_pos.and_then(|next_pos| {
                pos.single_step_dir_to(*next_pos, gtx.board_dim)
                    .and_then(|dir| pos.explicit_wrapping_translate(dir, 1, gtx.board_dim).1.then_some(dir))
            });

            let radius = gtx.cell_dim.side / 2.5;

            builder.circle(DrawMode::fill(), dest, radius, 0.1, Color::WHITE)?;
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

                builder.polygon(DrawMode::fill(), &points, Color::WHITE)?;
                stats.polygons += 1;
            }

            Ok(())
        });

    if let Err(e) = res {
        return Some(Err(e).with_trace_step("player_path_mesh"));
    }

    let mesh = Mesh::from_data(ctx, builder.build());
    Some(Ok(mesh))
}
