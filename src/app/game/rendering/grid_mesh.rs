use crate::{
    app::palette::GamePalette,
    basic::{CellDim, HexDim, Point},
};
use ggez::{
    graphics::{DrawMode, Mesh, MeshBuilder},
    Context, GameResult,
};
use num_integer::Integer;
use crate::app::game::Game;

impl Game {
    // TODO: make this readable
    pub(in crate::app::game) fn grid_mesh(
        &self,
        ctx: &mut Context,
    ) -> GameResult<Mesh> {
        let CellDim { side, sin, cos } = self.cell_dim;

        // two kinds of alternating vertical lines
        let mut vline_a = vec![]; // lines that start from the top with /
        let mut vline_b = vec![]; // lines that start from the top with \

        #[rustfmt::skip]
        for dv in (0..=self.dim.v).map(|v| v as f32 * 2. * sin) {
            vline_a.push(Point { x: cos, y: dv });
            vline_a.push(Point { x: 0., y: dv + sin });
            vline_b.push(Point { x: cos + side, y: dv });
            vline_b.push(Point { x: 2. * cos + side, y: dv + sin });
        }

        let mut builder = MeshBuilder::new();

        let draw_mode = DrawMode::stroke(self.palette.grid_thickness);
        let color = self.palette.grid_color;
        for h in 0..(self.dim.h + 1) / 2 {
            if h == 0 {
                builder.polyline(draw_mode, &vline_a[..vline_a.len() - 1], color)?;
            } else {
                builder.polyline(draw_mode, &vline_a, color)?;
            }
            if self.dim.h.is_odd() && h == (self.dim.h + 1) / 2 - 1 {
                builder.polyline(draw_mode, &vline_b[..vline_b.len() - 1], color)?;
            } else {
                builder.polyline(draw_mode, &vline_b, color)?;
            }

            let dh = h as f32 * (2. * side + 2. * cos);

            for v in 0..=self.dim.v {
                let dv = v as f32 * 2. * sin;

                // line between a and b
                builder.line(
                    #[rustfmt::skip] &[
                        Point { x: cos + dh, y: dv },
                        Point { x: cos + side + dh, y: dv },
                    ],
                    self.palette.grid_thickness,
                    color,
                )?;

                // line between b and a
                if !(self.dim.h.is_odd() && h == (self.dim.h + 1) / 2 - 1) {
                    builder.line(
                        #[rustfmt::skip] &[
                            Point { x: 2. * cos + side + dh, y: sin + dv },
                            Point { x: 2. * cos + 2. * side + dh, y: sin + dv },
                        ],
                        self.palette.grid_thickness,
                        color,
                    )?;
                }
            }

            // shift the lines right by 2 cells
            let offset = 2. * (side + cos);
            vline_a.iter_mut().for_each(|a| a.x += offset);
            vline_b.iter_mut().for_each(|b| b.x += offset);
        }
        if self.dim.h.is_even() {
            builder.polyline(draw_mode, &vline_a[1..], color)?;
        }

        builder.build(ctx)
    }

    pub(in crate::app::game) fn border_mesh(&self, _ctx: &mut Context) -> GameResult<Mesh> {
        // let _ = ctx;
        unimplemented!()
    }
}
