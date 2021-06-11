use crate::app::game::{Game, AppleType};
use ggez::{GameResult, Context};
use ggez::graphics::{Mesh, MeshBuilder, DrawMode};
use hsl::HSL;
use crate::basic::DrawStyle;
use crate::app::snake::rendering::render_hexagon;
use crate::basic::transformations::translate;
use ggez::graphics::Color;

impl Game {
    pub(in crate::app::game) fn apple_mesh(&self, ctx: &mut Context) -> GameResult<Mesh> {
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