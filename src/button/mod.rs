use crate::basic::Point;
use crate::button::Delta::{Changed, Unchanged};
use crate::color::Color;
use crate::error::{Error, ErrorConversion, Result};
use crate::rendering::shape::collisions::shape_point;
use crate::rendering::shape::{ShapePoints, ShapePointsSlice};
use ggez::event::MouseButton;
use ggez::graphics::{Canvas, DrawMode, DrawParam, Mesh, MeshBuilder, PxScale, Text, TextLayout};
use ggez::Context;
use std::error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Copy, Clone, Eq, PartialEq)]
enum State {
    Normal,
    Hover,
    JustClicked,
    Clicked,
}

#[derive(Copy, Clone, Debug)]
pub struct TriColor {
    pub normal: Color,
    pub hover: Color,
    pub click: Color,
}

impl TriColor {
    fn get(&self, state: State) -> Color {
        match state {
            State::Normal => self.normal,
            State::Hover => self.hover,
            State::JustClicked | State::Clicked => self.click,
        }
    }
}

// Very similar to `MessageDrawable`
#[derive(Debug, Clone)]
struct ButtonText {
    text: Text,
    relative_pos: Point,
    color: TriColor,
}

impl ButtonText {
    pub fn draw(&self, canvas: &mut Canvas, state: State) {
        let dp = DrawParam::default()
            .dest(self.relative_pos)
            .color(self.color.get(state));
        canvas.draw(&self.text, dp)
    }
}

#[derive(Debug, Clone)]
struct ButtonShape {
    // at origin
    base_points: ShapePoints,
    // relative to the button's overall location
    relative_pos: Point,
    // absolute position
    cached_points: Option<ShapePoints>,

    stroke_thickness: f32,
    color: TriColor,
}

impl ButtonShape {
    fn build(
        &mut self,
        builder: &mut MeshBuilder,
        absolute_pos: Delta<Point>,
        state: State,
    ) -> Result {
        use Delta::*;

        let points = match absolute_pos {
            Changed(pos) => self
                .cached_points
                .insert(self.base_points.clone().translate(pos + self.relative_pos)),
            Unchanged(_) => self.cached_points.as_ref().unwrap(),
        };

        let draw_mode = DrawMode::stroke(self.stroke_thickness);
        builder
            .polygon(draw_mode, points, *self.color.get(state))
            .map(|_| ())
            .map_err(Error::from)
            .with_trace_step("ButtonShape::build")
    }
}

#[derive(Debug, Clone)]
pub struct ButtonData {
    outer_shape: ButtonShape,
    inner_shapes: Vec<ButtonShape>,
    text: Option<ButtonText>,
    // TODO: cache hover/click
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Delta<T> {
    Changed(T),
    Unchanged(T),
}

impl<T> Delta<T> {
    fn into(self) -> T {
        match self {
            Changed(x) => x,
            Unchanged(x) => x,
        }
    }
}

impl ButtonData {
    fn draw(
        &mut self,
        canvas: &mut Canvas,
        ctx: &Context,
        absolute_pos: Delta<Point>,
        state: State,
    ) -> Result {
        let res: Result = try {
            if let Some(text) = &self.text {
                ButtonText {
                    relative_pos: text.relative_pos + absolute_pos.into(),
                    ..text.clone()
                }
                .draw(canvas, state)
            }

            let builder = &mut MeshBuilder::new();
            self.outer_shape.build(builder, absolute_pos, state)?;
            self.inner_shapes
                .iter_mut()
                .try_for_each(|bs| bs.build(builder, absolute_pos, state))?;

            let draw_param = DrawParam::default();
            canvas.draw(&Mesh::from_data(ctx, builder.build()), draw_param);
        };

        res.with_trace_step("ButtonData::build")
    }
}

#[derive(Debug, Clone)]
pub struct ButtonDataBuilder {
    outer_shape: Option<ButtonShape>,
    inner_shapes: Vec<ButtonShape>,
    text: Option<ButtonText>,
}

#[derive(Debug)]
pub struct ButtonDataBuilderError(pub Box<ButtonDataBuilder>, pub &'static str);

impl Display for ButtonDataBuilderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl error::Error for ButtonDataBuilderError {}

impl ButtonDataBuilder {
    pub fn new() -> Self {
        Self {
            outer_shape: None,
            inner_shapes: vec![],
            text: None,
        }
    }

    pub fn outer_shape(
        mut self,
        points: ShapePoints,
        stroke_thickness: f32,
        color: TriColor,
    ) -> Self {
        self.outer_shape = Some(ButtonShape {
            base_points: points,
            relative_pos: Point::zero(),
            cached_points: None,
            stroke_thickness,
            color,
        });
        self
    }

    pub fn inner_shape(
        mut self,
        points: ShapePoints,
        stroke_thickness: f32,
        relative_pos: Point,
        color: TriColor,
    ) -> Self {
        self.inner_shapes.push(ButtonShape {
            base_points: points,
            relative_pos,
            cached_points: None,
            stroke_thickness,
            color,
        });
        self
    }

    pub fn text(
        mut self,
        text: &str,
        font_size: f32,
        layout: TextLayout,
        relative_pos: Point,
        color: TriColor,
    ) -> Self {
        let mut text = Text::new(text);
        text
            // .set_font("arial")
            .set_scale(PxScale::from(font_size))
            .set_layout(layout);

        self.text = Some(ButtonText { text, relative_pos, color });
        self
    }

    pub fn build(self) -> Result<ButtonData> {
        let Some(outer_shape) = self.outer_shape else {
            let res: Result<_> =
                Err(ButtonDataBuilderError(Box::new(self), "Outer shape missing").into());
            return res.with_trace_step("ButtonDataBuilder::build");
        };
        Ok(ButtonData {
            outer_shape,
            inner_shapes: self.inner_shapes,
            text: self.text,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ButtonType {
    Click(ButtonData),
    Rotate {
        options: Vec<ButtonData>,
        index: usize,
    },
}

pub struct Button {
    pub pos: Point,
    pub button_type: ButtonType,
}

impl Button {
    // not translated
    fn outer_shape(&self) -> &ShapePointsSlice {
        use ButtonType::*;
        match &self.button_type {
            Click(data) => &data.outer_shape.base_points,
            Rotate { options, index } => &options[*index].outer_shape.base_points,
        }
    }

    fn state(&self, ctx: &Context) -> State {
        let mouse_pos = Point::from(ctx.mouse.position()) - self.pos;
        let mouse_down = ctx.mouse.button_pressed(MouseButton::Left);
        let just_clicked = ctx.mouse.button_just_pressed(MouseButton::Left);
        let outer_shape = self.outer_shape();

        assert!(!(just_clicked && !mouse_down));

        let overlap = shape_point(&outer_shape, mouse_pos);
        if overlap {
            if just_clicked {
                State::JustClicked
            } else if mouse_down {
                State::Clicked
            } else {
                State::Hover
            }
        } else {
            State::Normal
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas, ctx: &Context) -> Result<bool> {
        use ButtonType::*;

        let state = self.state(ctx);
        let res: Result = try {
            match &mut self.button_type {
                // TODO: actually signal if position was changed
                Click(data) => data.draw(canvas, ctx, Changed(self.pos), state)?,
                Rotate { options, index } => {
                    options[*index].draw(canvas, ctx, Changed(self.pos), state)?
                }
            }
        };

        if state == State::JustClicked {
            match &mut self.button_type {
                Rotate { options, index } => *index = (*index + 1) % options.len(),
                _ => {}
            }
        }

        res.map(|()| state == State::JustClicked)
            .with_trace_step("Button::draw")
    }
}
