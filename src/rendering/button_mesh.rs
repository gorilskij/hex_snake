use crate::app::game_context::GameContext;
use crate::basic::Point;
use crate::color::Color;
use crate::error::{ErrorConversion, Result};
use crate::rendering::shape::collisions::shape_point;
use crate::rendering::shape::{Hexagon, Shape, TriangleArrowLeft};
use ggez::event::MouseButton;
use ggez::graphics::{DrawMode, Mesh, MeshBuilder};
use ggez::Context;
use std::f32::consts::TAU;

pub fn palette_changing_buttons_mesh(
    gtx: &GameContext,
    ctx: &Context,
    offset: Point,
) -> Result<(Mesh, bool, bool)> {
    let bottom_right = gtx.board_dim.to_cartesian(gtx.cell_dim);
    let bottom_right = Point {
        x: bottom_right.x + gtx.cell_dim.cos,
        y: bottom_right.y,
    };

    type Arrow = TriangleArrowLeft;

    let margin = 2. * gtx.cell_dim.side;
    let button_dim = gtx.cell_dim * 2.5;
    let arrow_dim = gtx.cell_dim * 2.5;
    let arrow_center = Arrow::center(arrow_dim);
    const STROKE_THICKNESS: f32 = 15.0;
    const COLOR: Color = Color::gray(0.5);
    const COLOR_HOVER: Color = Color::GREEN;
    const COLOR_CLICK: Color = Color::RED;

    let left_pos = Point { x: 0.0, y: bottom_right.y + margin };
    let left_arrow_pos = left_pos + button_dim.center() - arrow_center;

    let right_pos = Point {
        x: bottom_right.x - button_dim.side - 2. * button_dim.cos,
        y: bottom_right.y + margin,
    };
    let right_arrow_pos = right_pos + button_dim.center() - arrow_center;

    let builder = &mut MeshBuilder::new();
    let draw_mode = DrawMode::stroke(STROKE_THICKNESS);
    let mouse_pos = Point::from(ctx.mouse.position()) - offset;
    let clicked = ctx.mouse.button_pressed(MouseButton::Left);
    let just_clicked = ctx.mouse.button_just_pressed(MouseButton::Left);

    let clicked_left;
    let mut clicked_right = false;
    let res: Result<_> = try {
        // left button
        let mut points = Hexagon::points(button_dim);
        let collided = shape_point(&points, mouse_pos - left_pos);
        clicked_left = collided && just_clicked;
        points = points.translate(left_pos);
        let color = if collided {
            if clicked {
                COLOR_CLICK
            } else {
                COLOR_HOVER
            }
        } else {
            COLOR
        };
        builder.polygon(draw_mode, &points, *color)?;

        let points = Arrow::points(arrow_dim).translate(left_arrow_pos);
        builder.polygon(draw_mode, &points, *color)?;

        // right button
        let mut points = Hexagon::points(button_dim);
        let collided = shape_point(&points, mouse_pos - right_pos);
        clicked_right = collided && just_clicked;
        points = points.translate(right_pos);
        let color = if collided {
            if clicked {
                COLOR_CLICK
            } else {
                COLOR_HOVER
            }
        } else {
            COLOR
        };
        builder.polygon(draw_mode, &points, *color)?;

        let points = Arrow::points(arrow_dim)
            .rotate_clockwise(arrow_center, TAU / 2.)
            .translate(right_arrow_pos);
        builder.polygon(draw_mode, &points, *color)?;

        builder.build()
    };

    res.map(|mesh_data| (Mesh::from_data(ctx, mesh_data), clicked_left, clicked_right))
        .with_trace_step("palette_changing_buttons_mesh")
}
