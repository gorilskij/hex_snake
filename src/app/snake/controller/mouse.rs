use crate::app::snake::controller::Controller;
use crate::app::snake::Body;
use crate::app::snake::utils::OtherSnakes;
use crate::app::apple::Apple;
use crate::basic::{Dir, Point, CellDim, HexPoint};
use ggez::Context;
use ggez::input::mouse;
use std::f32::consts::TAU;
use crate::app::rendering::segments::render_hexagon;
use crate::basic::transformations::translate;
use ggez::graphics::{Mesh, DrawMode, StrokeOptions, Color};
use crate::app::game_context::GameContext;
use crate::app::app_error::{AppResult, GameResultExtension};

pub struct Mouse;

impl Controller for Mouse {
    fn next_dir(&mut self, body: &mut Body, other_snakes: OtherSnakes, apples: &[Apple], gtx: &GameContext, ctx: &Context) -> Option<Dir> {
        let mouse_position: Point = mouse::position(ctx).into();
        let target = HexPoint::from_cartesian(mouse_position, gtx.cell_dim);

        let current = body.cells[0].pos;

        // actual cell_dim doesn't matter, scaling preserves angles
        let CellDim { sin, .. } = CellDim::from(1.);
        let dx = (target.h - current.h) as f32;
        let dy = -(target.v - current.v) as f32 / (2. * sin);
        let angle = (dy.atan2(dx) + TAU) % TAU;
        let dir = Dir::closest_to_angle(angle)
            .into_iter()
            .filter(|dir| *dir != -body.dir)
            .next();

        dir
    }

    fn get_mesh(&self, gtx: &GameContext, ctx: &mut Context) -> Option<AppResult<Mesh>> {
        let mouse_position: Point = mouse::position(ctx).into();
        // let position = mouse_position - self.cell_dim.center();
        let position = HexPoint::from_cartesian(mouse_position, gtx.cell_dim).to_cartesian(gtx.cell_dim);
        let mut hexagon = render_hexagon(gtx.cell_dim);
        translate(&mut hexagon, position);
        let draw_mode = DrawMode::Stroke(
            StrokeOptions::default()
                .with_line_width(gtx.palette.border_thickness)
        );
        Some(Mesh::new_polygon(ctx, draw_mode, &hexagon, Color::CYAN).into_with_trace("controller::Mouse::get_mesh"))
    }
}
