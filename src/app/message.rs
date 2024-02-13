use crate::basic::Point;
use crate::color::Color;
use crate::support::text_layout::TextLayoutExtension;
use ggez::graphics::{Canvas, DrawParam, PxScale, Text, TextLayout};
use ggez::Context;
use std::time::{Duration, Instant};

/// Finite number of possible messages
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum MessageID {
    /// Persistent fps view
    Fps,
    /// Temporary info when resizing window,
    /// toggling grid, or other notifications
    Notification,
    /// Stats about the game
    Stats,
}

pub enum Position {
    TopLeft,
    TopRight,
}

pub struct Message {
    pub text: String,

    pub position: Position,
    pub h_margin: f32,
    pub v_margin: f32,
    pub font_size: f32,
    pub color: Color,
    // None means unlimited duration
    pub disappear: Option<Instant>,
}

impl Message {
    pub const DEFAULT_MARGIN: f32 = 20.;
    pub const DEFAULT_FONT_SIZE: f32 = 50.;

    // `layout` refers to where the text should be placed in the window
    pub fn default(
        text: String,
        position: Position,
        color: Color,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            text,
            position,
            h_margin: Self::DEFAULT_MARGIN,
            v_margin: Self::DEFAULT_MARGIN,
            font_size: Self::DEFAULT_FONT_SIZE,
            color,
            disappear: duration.map(|d| Instant::now() + d),
        }
    }
}

pub struct MessageDrawable {
    pub text: Text,
    pub dest: Point,
    pub color: Color,
}

impl MessageDrawable {
    pub fn draw(&self, canvas: &mut Canvas) {
        let dp = DrawParam::default().dest(self.dest).color(self.color);

        canvas.draw(&self.text, dp)
    }
}

impl Message {
    /// A return value of None signifies that the message has reached
    /// its end of life and should be removed
    pub fn get_drawable(&self, ctx: &Context) -> Option<MessageDrawable> {
        let (width, height) = ctx.gfx.drawable_size();

        let dest;
        let layout;
        match self.position {
            Position::TopLeft => {
                dest = Point { x: self.h_margin, y: self.v_margin };
                layout = TextLayout::top_left();
            }
            Position::TopRight => {
                dest = Point {
                    x: width - self.h_margin,
                    y: self.v_margin,
                };
                layout = TextLayout::top_right();
            }
        }

        // fade out
        let mut color = self.color;
        if let Some(deadline) = self.disappear {
            match deadline.checked_duration_since(Instant::now()) {
                None => return None, // Message has reached its end of life
                Some(time_left) => {
                    let millis = time_left.as_millis();
                    if millis < 200 {
                        // linear fade out
                        color.a = millis as f32 / 200.;
                    }
                }
            }
        }

        let mut text = Text::new(self.text.as_str());
        text
            // .set_font("arial")
            .set_scale(PxScale::from(self.font_size))
            .set_bounds([width / 2. - self.h_margin, height / 2. - self.v_margin])
            .set_layout(layout);

        Some(MessageDrawable { text, dest, color })
    }
}
