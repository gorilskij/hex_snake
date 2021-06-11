use crate::{
    app::{
        game::{AppleType, Game, Stats},
        snake::rendering::render_hexagon,
    },
    basic::{transformations::translate, DrawStyle},
};
use ggez::{
    graphics::{Color, DrawMode, Mesh, MeshBuilder},
    Context, GameResult,
};
use hsl::HSL;

impl Game {
    pub(in crate::app::game) fn apple_mesh(
        &self,
        ctx: &mut Context,
        stats: &mut Stats,
    ) -> GameResult<Mesh> {
        stats.redrawing_apples = true;

        let mut builder = MeshBuilder::new();

        for apple in &self.apples {
            let color = match apple.typ {
                AppleType::Normal(_) => self.palette.apple_color,
                AppleType::SpawnSnake(_) => {
                    let hue = 360. * (self.control.graphics_frame_num() as f64 / 60. % 1.);
                    let hsl = HSL { h: hue, s: 1., l: 0.3 };
                    Color::from(hsl.to_rgb())
                }
            };

            if self.prefs.draw_style == DrawStyle::Hexagon {
                let dest = apple.pos.to_point(self.cell_dim);
                let mut points = render_hexagon(self.cell_dim);
                translate(&mut points, dest);
                builder.polygon(DrawMode::fill(), &points, color)?;
            } else {
                let dest = apple.pos.to_point(self.cell_dim) + self.cell_dim.center();
                builder.circle(DrawMode::fill(), dest, self.cell_dim.side / 1.5, 0.1, color)?;
            }
        }

        builder.build(ctx)
    }
}
