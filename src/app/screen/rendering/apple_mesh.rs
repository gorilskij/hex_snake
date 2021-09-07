use crate::{
    app::{
        palette::Palette,
        screen::{
            {Apple, AppleType},
            stats::Stats,
        },
        snake::rendering::render_hexagon,
    },
    basic::{transformations::translate, CellDim, DrawStyle},
};
use ggez::{
    graphics::{Color, DrawMode, Mesh, MeshBuilder},
    Context, GameResult,
};
use hsl::HSL;
use crate::app::screen::control::FrameStamp;

pub(in crate::app::screen) fn get_apple_mesh(
    apples: &[Apple],
    frame_stamp: FrameStamp,
    cell_dim: CellDim,
    draw_style: DrawStyle,
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
        let color = match apple.typ {
            AppleType::Normal(_) => palette.apple_color,
            AppleType::SpawnSnake(_) => {
                let hue = 360. * (frame_stamp.0 as f64 / 60. % 1.);
                let hsl = HSL { h: hue, s: 1., l: 0.3 };
                Color::from(hsl.to_rgb())
            }
        };

        if draw_style == DrawStyle::Hexagon {
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
