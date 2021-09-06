use crate::{
    app::{
        snake::rendering::render_hexagon,
    },
    basic::{transformations::translate, DrawStyle},
};
use ggez::{
    graphics::{Color, DrawMode, Mesh, MeshBuilder},
    Context, GameResult,
};
use hsl::HSL;
use crate::app::screen::stats::Stats;
use crate::app::screen::game::{Game, AppleType, Apple};
use crate::app::screen::control::Control;
use crate::app::palette::Palette;
use crate::basic::CellDim;

pub(in crate::app::screen) fn get_apple_mesh(
    apples: &[Apple],
    control: &Control,
    cell_dim: CellDim,
    draw_style: DrawStyle,
    palette: &Palette,
    ctx: &mut Context,
    stats: &mut Stats,
    ) -> GameResult<Mesh> {
        stats.redrawing_apples = true;

        let mut builder = MeshBuilder::new();

        for apple in apples {
            let color = match apple.typ {
                AppleType::Normal(_) => palette.apple_color,
                AppleType::SpawnSnake(_) => {
                    let hue = 360. * (control.graphics_frame_num() as f64 / 60. % 1.);
                    let hsl = HSL { h: hue, s: 1., l: 0.3 };
                    Color::from(hsl.to_rgb())
                }
            };

            if draw_style == DrawStyle::Hexagon {
                let dest = apple.pos.to_point(cell_dim);
                let mut points = render_hexagon(cell_dim);
                translate(&mut points, dest);
                builder.polygon(DrawMode::fill(), &points, color)?;
            } else {
                let dest = apple.pos.to_point(cell_dim) + cell_dim.center();
                builder.circle(DrawMode::fill(), dest, cell_dim.side / 1.5, 0.1, color)?;
            }
            stats.polygons += 1;
        }

        builder.build(ctx)
    }
