use crate::{
    app::{
        app_error::{AppResult, GameResultExtension},
        apple::Apple,
        game_context::GameContext,
        rendering::segments::render_hexagon,
        snake::{controller::Controller, utils::OtherSnakes, Body},
    },
    basic::{transformations::translate, CellDim, Dir, HexPoint, Point},
};
use ggez::{
    graphics::{Color, DrawMode, Mesh, StrokeOptions},
    input::mouse,
    Context,
};
use std::f32::consts::TAU;

pub struct Mouse;

impl Controller for Mouse {
    fn next_dir(
        &mut self,
        body: &mut Body,
        _: OtherSnakes,
        _: &[Apple],
        gtx: &GameContext,
        ctx: &Context,
    ) -> Option<Dir> {
        let mouse_position: Point = mouse::position(ctx).into();
        let target = HexPoint::from_cartesian(mouse_position, gtx.cell_dim);

        let current = body.cells[0].pos;

        // actual cell_dim doesn't matter, scaling preserves angles
        let CellDim { sin, .. } = CellDim::from(1.);
        let dx = (target.h - current.h) as f32;
        let dy = -(target.v - current.v) as f32 / (2. * sin);
        let angle = (dy.atan2(dx) + TAU) % TAU;
        Dir::closest_to_angle(angle)
            .into_iter()
            .find(|dir| *dir != -body.dir)
    }

    fn get_mesh(&self, gtx: &GameContext, ctx: &mut Context) -> Option<AppResult<Mesh>> {
        let mouse_position: Point = mouse::position(ctx).into();
        // let position = mouse_position - self.cell_dim.center();
        let position =
            HexPoint::from_cartesian(mouse_position, gtx.cell_dim).to_cartesian(gtx.cell_dim);
        let mut hexagon = render_hexagon(gtx.cell_dim);
        translate(&mut hexagon, position);
        let draw_mode = DrawMode::Stroke(
            StrokeOptions::default().with_line_width(gtx.palette.border_thickness),
        );
        Some(
            Mesh::new_polygon(ctx, draw_mode, &hexagon, Color::CYAN)
                .into_with_trace("controller::Mouse::get_mesh"),
        )
    }
}
