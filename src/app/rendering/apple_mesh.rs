use ggez::{
    graphics::{Color, DrawMode, Mesh, MeshBuilder},
    Context, GameResult,
};
use hsl::HSL;

use crate::{
    app::{
        apple::{self, Apple},
        palette::Palette,
        rendering,
        rendering::segments::render_hexagon,
        stats::Stats,
    },
    basic::{transformations::translate, CellDim, FrameStamp},
};
use crate::app::game_context::GameContext;

pub fn apple_mesh(
    apples: &[Apple],
    gtx: &GameContext,
    ctx: &mut Context,
    stats: &mut Stats,
) -> GameResult<Mesh> {
    if apples.is_empty() {
        panic!("tried to draw a mesh with 0 apples")
    }

    stats.redrawing_apples = true;

    let mut builder = MeshBuilder::new();

    for apple in apples {
        let color = match apple.apple_type {
            apple::Type::Normal(_) => gtx.palette.apple_color,
            apple::Type::SpawnSnake(_) => {
                let hue = 360. * (gtx.frame_stamp.0 as f64 / 60. % 1.);
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

    builder.build(ctx)
}
