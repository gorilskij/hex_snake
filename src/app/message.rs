use crate::basic::Point;
use ggez::{
    graphics::{self, Color, DrawParam, Font, PxScale, Text},
    Context,
};
use std::time::{Duration, Instant};
use crate::app::app_error::{AppResult, GameResultExtension};

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

pub struct Message {
    pub text: String,

    pub left: bool,
    pub top: bool,
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

    pub fn default_top_left(text: String, color: Color, duration: Option<Duration>) -> Self {
        Self {
            text,
            left: true,
            top: true,
            h_margin: Self::DEFAULT_MARGIN,
            v_margin: Self::DEFAULT_MARGIN,
            font_size: Self::DEFAULT_FONT_SIZE,
            color,
            disappear: duration.map(|d| Instant::now() + d),
        }
    }

    pub fn default_top_right(text: String, color: Color, duration: Option<Duration>) -> Self {
        Self {
            text,
            left: false,
            top: true,
            h_margin: Self::DEFAULT_MARGIN,
            v_margin: Self::DEFAULT_MARGIN,
            font_size: Self::DEFAULT_FONT_SIZE,
            color,
            disappear: duration.map(|d| Instant::now() + d),
        }
    }

    /// Returns Ok(true) if the message should continue existing
    /// and Ok(false) if it should be removed
    pub fn draw(&self, ctx: &mut Context) -> AppResult<bool> {
        let mut text = Text::new(self.text.as_str());
        text.set_font(Font::default(), PxScale::from(self.font_size));

        let (width, height) = graphics::drawable_size(ctx);

        let x = if self.left {
            self.h_margin
        } else {
            width - text.width(ctx) - self.h_margin
        };

        let y = if self.top {
            self.v_margin
        } else {
            height - text.height(ctx) - self.v_margin
        };

        let location = Point { x, y };

        // fade out
        let mut color = self.color;
        if let Some(deadline) = self.disappear {
            match deadline.checked_duration_since(Instant::now()) {
                None => return Ok(false), // Message has reached its end of life
                Some(time_left) => {
                    let millis = time_left.as_millis();
                    if millis < 200 {
                        // linear fade out
                        color.a = millis as f32 / 200.;
                    }
                }
            }
        }

        ggez::graphics::draw(ctx, &text, DrawParam::from((location, color))).into_with_trace("Message::draw")?;
        Ok(true)
    }
}
