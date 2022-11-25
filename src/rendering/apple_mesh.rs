use ggez::{
    graphics::{Color, DrawMode, Mesh, MeshBuilder},
    Context,
};
use hsl::HSL;

use crate::{
    app::{game_context::GameContext, stats::Stats},
    apple::{self, Apple},
    basic::transformations::translate,
    error::{AppErrorConversion, AppResult},
    rendering,
    rendering::segments::render_hexagon,
};

pub fn apple_mesh(
    apples: &[Apple],
    gtx: &GameContext,
    ctx: &mut Context,
    stats: &mut Stats,
) -> AppResult<Mesh> {
    if apples.is_empty() {
        panic!("tried to draw a mesh with 0 apples")
    }

    stats.redrawing_apples = true;

    let mut builder = MeshBuilder::new();

    let res: AppResult<Mesh> = try {
        for apple in apples {
            use crate::apple::Type::*;
            let color = match apple.apple_type {
                Food(_) => gtx.palette.apple_color,
                SpawnSnake(_) | SpawnRain => {
                    let hue = 360. * (gtx.elapsed_millis as f64 / 1000. % 1.);
                    let hsl = HSL { h: hue, s: 1., l: 0.3 };
                    Color::from(hsl.to_rgb())
                }
            };

            if gtx.prefs.draw_style == rendering::Style::Hexagon {
                let dest = apple.pos.to_cartesian(gtx.cell_dim);
                let mut points = render_hexagon(gtx.cell_dim);
                translate(&mut points, dest);
                builder.polygon(DrawMode::fill(), &points, color)?;
            } else {
                let dest = apple.pos.to_cartesian(gtx.cell_dim) + gtx.cell_dim.center();
                builder.circle(DrawMode::fill(), dest, gtx.cell_dim.side / 1.5, 0.1, color)?;
            }
            stats.polygons += 1;
        }

        builder.build(ctx)?
    };
    res.with_trace_step("apple_mesh")
}
