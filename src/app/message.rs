use std::time::Duration;

use macroquad::camera::set_default_camera;
use macroquad::text::{draw_text, measure_text};
use macroquad::window::screen_width;

use crate::color::Color;
use crate::gfx::graphics::Canvas;
use crate::gfx::time::Instant;

/// Finite number of possible messages
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum MessageID {
    /// Persistent fps view
    Fps,
    /// Temporary info when resizing window, toggling grid, or other notifications
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
    pub const DEFAULT_FONT_SIZE: f32 = 40.;

    pub fn default(text: String, position: Position, color: Color, duration: Option<Duration>) -> Self {
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
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub font_size: u16,
    pub color: Color,
}

impl MessageDrawable {
    pub fn draw(&self, _canvas: &mut Canvas) {
        // text lives in screen space, not the board-offset camera
        set_default_camera();
        let color = *self.color;
        draw_text(&self.text, self.x, self.y, self.font_size as f32, color.into());
    }
}

impl Message {
    /// A return value of None signifies that the message has reached its end of
    /// life and should be removed.
    pub fn get_drawable(&self) -> Option<MessageDrawable> {
        let screen_w = screen_width();

        // fade out
        let mut color = self.color;
        if let Some(deadline) = self.disappear {
            match deadline.checked_duration_since(Instant::now()) {
                None => return None, // Message has reached its end of life
                Some(time_left) => {
                    let millis = time_left.as_millis();
                    if millis < 200 {
                        color.a = millis as f32 / 200.;
                    }
                }
            }
        }

        let font_size = self.font_size as u16;
        let dims = measure_text(&self.text, None, font_size, 1.0);
        let (x, y) = match self.position {
            Position::TopLeft => (self.h_margin, self.v_margin + dims.offset_y),
            Position::TopRight => (screen_w - self.h_margin - dims.width, self.v_margin + dims.offset_y),
        };

        Some(MessageDrawable {
            text: self.text.clone(),
            x,
            y,
            font_size,
            color,
        })
    }
}
