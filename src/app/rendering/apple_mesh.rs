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

pub fn apple_mesh(
    apples: &[Apple],
    frame_stamp: FrameStamp,
    cell_dim: CellDim,
    draw_style: rendering::Style,
    palette: &Palette,
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
            apple::Type::Normal(_) => palette.apple_color,
            apple::Type::SpawnSnake(_) => {
                let hue = 360. * (frame_stamp.0 as f64 / 60. % 1.);
                let hsl = HSL { h: hue, s: 1., l: 0.3 };
                Color::from(hsl.to_rgb())
            }
        };

        if draw_style == rendering::Style::Hexagon {
            let dest = apple.pos.to_cartesian(cell_dim);
            let mut points = render_hexagon(cell_dim);
            translate(&mut points, dest);
            builder.polygon(DrawMode::fill(), &points, color)?;
        } else {
            let dest = apple.pos.to_cartesian(cell_dim) + cell_dim.center();
            builder.circle(DrawMode::fill(), dest, cell_dim.side / 1.5, 0.1, color)?;
        }
        stats.polygons += 1;
    }

    builder.build(ctx)
}
