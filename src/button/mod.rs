//! Polygon-shaped buttons drawn with the board's own meshes.
//!
//! A button is an outer outline (e.g. a hexagon) plus optional inner shapes
//! (icons) and a text label, all colored by the button's [`State`]. Hit
//! testing uses the exact outline, not its bounding box. Buttons are drawn in
//! screen space: the caller sets the default camera first.

use macroquad::color::Color;
use macroquad::input::{is_mouse_button_down, is_mouse_button_pressed, mouse_position, MouseButton};
use macroquad::text::{draw_text, measure_text};

use crate::basic::Point;
use crate::rendering::shape::collisions::shape_point;
use crate::rendering::shape::ShapePoints;
use crate::support::mesh::{build_polygon, DrawMode, Mesh};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum State {
    Normal,
    Hover,
    /// The mouse is held down over the button
    Pressed,
}

/// One color per [`State`].
#[derive(Copy, Clone, Debug)]
pub struct TriColor {
    pub normal: Color,
    pub hover: Color,
    pub pressed: Color,
}

impl TriColor {
    fn get(&self, state: State) -> Color {
        match state {
            State::Normal => self.normal,
            State::Hover => self.hover,
            State::Pressed => self.pressed,
        }
    }
}

#[derive(Clone, Debug)]
struct ButtonShape {
    /// Relative to the button's position
    points: ShapePoints,
    stroke_thickness: f32,
    color: TriColor,
}

#[derive(Clone, Debug)]
struct ButtonText {
    text: String,
    font_size: f32,
    /// Where the text is centered, relative to the button's position
    center: Point,
    color: TriColor,
}

/// The look of a button: what [`Button`] draws in each state.
#[derive(Clone, Debug)]
pub struct ButtonData {
    outer_shape: ButtonShape,
    inner_shapes: Vec<ButtonShape>,
    text: Option<ButtonText>,
}

impl ButtonData {
    /// A button outlined by `points` (relative to the button's position).
    pub fn new(points: ShapePoints, stroke_thickness: f32, color: TriColor) -> Self {
        Self {
            outer_shape: ButtonShape { points, stroke_thickness, color },
            inner_shapes: vec![],
            text: None,
        }
    }

    /// Add an outlined shape, e.g. an icon, drawn `offset` from the button's
    /// position.
    pub fn inner_shape(mut self, points: ShapePoints, offset: Point, stroke_thickness: f32, color: TriColor) -> Self {
        self.inner_shapes.push(ButtonShape {
            points: points.translate(offset),
            stroke_thickness,
            color,
        });
        self
    }

    /// Add a text label centered in the outer shape.
    pub fn text(mut self, text: impl Into<String>, font_size: f32, color: TriColor) -> Self {
        self.text = Some(ButtonText {
            text: text.into(),
            font_size,
            center: self.outer_shape.points.center(),
            color,
        });
        self
    }

    fn draw(&self, pos: Point, state: State) {
        let shapes = std::iter::once(&self.outer_shape).chain(&self.inner_shapes);
        let mesh = Mesh::combine(shapes.map(|shape| {
            let points = shape.points.clone().translate(pos);
            build_polygon(DrawMode::stroke(shape.stroke_thickness), &points, shape.color.get(state))
        }));
        mesh.draw();

        if let Some(text) = &self.text {
            let font_size = text.font_size as u16;
            let dims = measure_text(&text.text, None, font_size, 1.);
            let center = pos + text.center;
            // draw_text's y is the baseline
            let x = center.x - dims.width / 2.;
            let y = center.y - dims.height / 2. + dims.offset_y;
            draw_text(&text.text, x, y, text.font_size, text.color.get(state));
        }
    }
}

#[derive(Clone, Debug)]
pub enum ButtonType {
    Click(ButtonData),
    /// Cycles through `options` on each click
    Rotate { options: Vec<ButtonData>, index: usize },
}

pub struct Button {
    /// Top-left of the button's shapes, in screen coordinates
    pub pos: Point,
    pub button_type: ButtonType,
}

impl Button {
    pub fn click(pos: Point, data: ButtonData) -> Self {
        Self { pos, button_type: ButtonType::Click(data) }
    }

    pub fn rotate(pos: Point, options: Vec<ButtonData>) -> Self {
        assert!(!options.is_empty(), "a rotate button needs at least one option");
        Self {
            pos,
            button_type: ButtonType::Rotate { options, index: 0 },
        }
    }

    fn data(&self) -> &ButtonData {
        match &self.button_type {
            ButtonType::Click(data) => data,
            ButtonType::Rotate { options, index } => &options[*index],
        }
    }

    /// Which option a rotate button is showing (always 0 for a click button).
    pub fn index(&self) -> usize {
        match self.button_type {
            ButtonType::Click(_) => 0,
            ButtonType::Rotate { index, .. } => index,
        }
    }

    /// Width and height of the button's outer shape
    pub fn size(&self) -> Point {
        let (min, max) = self.data().outer_shape.points.bounding_box();
        max - min
    }

    pub fn is_hovered(&self) -> bool {
        let (x, y) = mouse_position();
        let mouse = Point { x, y } - self.pos;
        shape_point(&self.data().outer_shape.points, mouse)
    }

    /// Draw the button and report whether it was clicked this frame. A click
    /// advances a rotate button to its next option.
    pub fn draw(&mut self) -> bool {
        let hovered = self.is_hovered();
        let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

        if clicked {
            if let ButtonType::Rotate { options, index } = &mut self.button_type {
                *index = (*index + 1) % options.len();
            }
        }

        let state = match (hovered, is_mouse_button_down(MouseButton::Left)) {
            (false, _) => State::Normal,
            (true, false) => State::Hover,
            (true, true) => State::Pressed,
        };
        self.data().draw(self.pos, state);

        clicked
    }
}
