use crate::app::game_context::GameContext;
use crate::app::message::MessageDrawable;
use crate::basic::{CellDim, Point};
use crate::color::Color;
use crate::error::{ErrorConversion, Result};
use crate::rendering::shape::collisions::shape_point;
use crate::rendering::shape::{Hexagon, Shape, TriangleArrowLeft, WideHexagon};
use ggez::event::MouseButton;
use ggez::graphics::{DrawMode, Mesh, MeshBuilder, PxScale, Text, TextLayout};
use ggez::Context;
use std::f32::consts::TAU;

const DIM_MULTIPLIER: f32 = 2.;
const STROKE_THICKNESS: f32 = 15.0;
const COLOR: Color = Color::gray(0.5);
const COLOR_HOVER: Color = Color::GREEN;
const COLOR_CLICK: Color = Color::RED;

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

    type Outline = Hexagon;
    type Arrow = TriangleArrowLeft;

    let margin = 2. * gtx.cell_dim.side;
    let button_dim = gtx.cell_dim * DIM_MULTIPLIER;
    let arrow_dim = gtx.cell_dim * DIM_MULTIPLIER;
    let arrow_center = Arrow::center(arrow_dim);

    let side = gtx.cell_dim.side;

    let left_pos = Point {
        x: 3. * side,
        y: bottom_right.y + margin,
    };
    let left_arrow_pos = left_pos + button_dim.center() - arrow_center;

    let right_pos = Point {
        x: bottom_right.x - button_dim.side - 2. * button_dim.cos - 3. * side,
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
        let points = Outline::points(button_dim).translate(left_pos);
        let collided = shape_point(&points, mouse_pos);
        clicked_left = collided && just_clicked;
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
        let points = Outline::points(button_dim).translate(right_pos);
        let collided = shape_point(&points, mouse_pos);
        clicked_right = collided && just_clicked;
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

pub fn player_number_buttons_mesh(
    cell_dim: CellDim,
    ctx: &Context,
) -> Result<(
    Mesh,
    bool,
    bool,
    Option<MessageDrawable>,
    Option<MessageDrawable>,
)> {
    let button_dim = cell_dim * 1.5;
    const STROKE_THICKNESS: f32 = 15.0;
    const COLOR: Color = Color::gray(0.5);
    const COLOR_HOVER: Color = Color::GREEN;
    const COLOR_CLICK: Color = Color::RED;
    const FONT_SIZE: f32 = 50.;

    let single_pos = Point { x: 400., y: 50. };
    let double_pos = Point { x: 1200., y: 50. };

    let builder = &mut MeshBuilder::new();
    let draw_mode = DrawMode::stroke(STROKE_THICKNESS);
    let mouse_pos = Point::from(ctx.mouse.position());
    let clicked = ctx.mouse.button_pressed(MouseButton::Left);
    let just_clicked = ctx.mouse.button_just_pressed(MouseButton::Left);

    let clicked_single;
    let mut clicked_double = false;

    let mut message_single = None;
    let mut message_double = None;

    let res: Result<_> = try {
        // single button
        let points = WideHexagon::points(button_dim).translate(single_pos);
        let collided = shape_point(&points, mouse_pos);
        clicked_single = collided && just_clicked;
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

        let mut text = Text::new("Single player");
        let message_pos = single_pos + WideHexagon::center(button_dim);
        text
            // .set_font("arial")
            .set_scale(PxScale::from(FONT_SIZE))
            .set_layout(TextLayout::center());
        message_single = Some(MessageDrawable { text, dest: message_pos, color });

        // double button
        let mut points = WideHexagon::points(button_dim);
        let collided = shape_point(&points, mouse_pos - double_pos);
        clicked_double = collided && just_clicked;
        points = points.translate(double_pos);
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

        let mut text = Text::new("2-player");
        let message_pos = double_pos + WideHexagon::center(button_dim);
        text
            // .set_font("arial")
            .set_scale(PxScale::from(FONT_SIZE))
            .set_layout(TextLayout::center());
        message_double = Some(MessageDrawable { text, dest: message_pos, color });

        builder.build()
    };

    res.map(|mesh_data| {
        (
            Mesh::from_data(ctx, mesh_data),
            clicked_single,
            clicked_double,
            message_single,
            message_double,
        )
    })
    .with_trace_step("player_number_buttons_mesh")
}
