use ggez::graphics::{Color, DrawMode, Mesh, MeshBuilder};
use ggez::Context;
use hsl::HSL;

use crate::app::fps_control::FpsContext;
use crate::app::game_context::GameContext;
use crate::app::stats::Stats;
use crate::apple::Apple;
use crate::error::{ErrorConversion, Result};
use crate::rendering;
use crate::rendering::shape::{Hexagon, Shape};

pub fn apple_mesh(
    apples: &[Apple],
    gtx: &GameContext,
    ftx: &FpsContext,
    ctx: &Context,
    stats: &mut Stats,
) -> Result<Mesh> {
    assert!(!apples.is_empty(), "tried to draw a mesh with 0 apples");

    stats.redrawing_apples = true;

    let mut builder = MeshBuilder::new();

    let res: Result<Mesh> = try {
        for apple in apples {
            use crate::apple::Type::*;
            let color = match apple.apple_type {
                Food(_) => gtx.palette.apple_color,
                SpawnSnake(_) | SpawnRain => {
                    let hue = 360. * (ftx.elapsed_millis as f64 / 1000. % 1.);
                    let hsl = HSL { h: hue, s: 1., l: 0.3 };
                    Color::from(hsl.to_rgb())
                }
            };

            match gtx.prefs.draw_style {
                rendering::Style::Hexagon => {
                    let dest = apple.pos.to_cartesian(gtx.cell_dim);
                    let points = Hexagon::new(gtx.cell_dim).translate(dest);
                    builder.polygon(DrawMode::fill(), &points, color)?;
                    stats.polygons += 1;
                }
                rendering::Style::Smooth => {
                    let dest = apple.pos.to_cartesian(gtx.cell_dim) + gtx.cell_dim.center();
                    let radius = gtx.cell_dim.side / 2.;
                    builder.circle(DrawMode::fill(), dest, radius, 0.1, color)?;
                    stats.polygons += 1;
                }
            }
        }

        Mesh::from_data(ctx, builder.build())
    };
    res.with_trace_step("apple_mesh")
}
