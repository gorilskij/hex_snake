use crate::app::game_context::GameContext;
use crate::app::stats::Stats;
use crate::apple::Apple;
use crate::basic::transformations::{rotate_clockwise, translate};
use crate::basic::{Dir, Point};
use crate::error::{Error, ErrorConversion, Result};
use crate::snake::{PassthroughKnowledge, Snake};
use crate::view::snakes::OtherSnakes;
use ggez::graphics::{Color, DrawMode, Mesh, MeshBuilder};
use ggez::Context;
use std::f32::consts::PI;
use std::iter;

pub fn player_path_mesh(
    player_snake: &mut Snake,
    other_snakes: OtherSnakes,
    apples: &[Apple],
    ctx: &mut Context,
    gtx: &GameContext,
    stats: &mut Stats,
) -> Option<Result<Mesh>> {
    player_snake.autopilot.as_mut().map(|autopilot| {
        let mut builder = MeshBuilder::new();
        // TODO: this conversion is too expensive
        let passthrough_knowledge = PassthroughKnowledge::accurate(&player_snake.eat_mechanics);
        let path = autopilot
            .get_path(
                &player_snake.body,
                Some(&passthrough_knowledge),
                &other_snakes,
                apples,
                gtx,
            )
            .expect("autopilot didn't provide path");

        for (pos, next_pos) in path
            .iter()
            .zip(path.iter().skip(1).map(Some).chain(iter::once(None)))
        {
            let dest = pos.to_cartesian(gtx.cell_dim) + gtx.cell_dim.center();

            // for the last point before a teleport, display a subtle hint about which direction
            // the snake should be going to teleport correctly
            let arrow = next_pos.and_then(|next_pos| {
                pos.single_step_dir_to(*next_pos, gtx.board_dim)
                    .and_then(|dir| {
                        pos.explicit_wrapping_translate(dir, 1, gtx.board_dim)
                            .1
                            .then_some(dir)
                    })
            });

            let radius = gtx.cell_dim.side / 2.5;

            builder.circle(DrawMode::fill(), dest, radius, 0.1, Color::WHITE)?;
            stats.polygons += 1;

            if let Some(dir) = arrow {
                // pointing down
                let mut points = vec![
                    Point { x: 0.0, y: radius * 2_f32.sqrt() },
                    Point {
                        x: radius * (PI / 4.).cos(),
                        y: radius * (PI / 4.).sin(),
                    },
                    Point {
                        x: -radius * (PI / 4.).cos(),
                        y: radius * (PI / 4.).sin(),
                    },
                ];
                rotate_clockwise(&mut points, Point::zero(), Dir::D.clockwise_angle_to(dir));
                translate(&mut points, dest);

                builder.polygon(DrawMode::fill(), &points, Color::WHITE)?;
                stats.polygons += 1;
            }
        }

        builder
            .build(ctx)
            .map_err(Error::from)
            .with_trace_step("player_path_mesh")
    })
}
