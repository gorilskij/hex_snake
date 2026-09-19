use std::time::Duration;

use macroquad::color::Color;
use macroquad::shapes::draw_rectangle;
use macroquad::window::screen_width;

use crate::support::text::{draw_text, measure_text};
use crate::support::time::Instant;

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
    /// Whether to dim the board behind the text
    pub background: bool,
}

impl Message {
    pub const DEFAULT_MARGIN: f32 = 20.;
    pub const DEFAULT_FONT_SIZE: f32 = 34.;

    pub fn default(text: String, position: Position, color: Color, duration: Option<Duration>) -> Self {
        Self {
            text,
            position,
            h_margin: Self::DEFAULT_MARGIN,
            v_margin: Self::DEFAULT_MARGIN,
            font_size: Self::DEFAULT_FONT_SIZE,
            color,
            disappear: duration.map(|d| Instant::now() + d),
            background: false,
        }
    }
}

/// Distance between lines' baselines, relative to the font size
const LINE_HEIGHT: f32 = 1.25;
/// Space between a message's text and the edge of its background, relative
/// to the font size
const BACKGROUND_PADDING: f32 = 0.3;
const BACKGROUND_ALPHA: f32 = 0.5;
/// How far letters reach below the baseline, relative to the font size
const DESCENT: f32 = 0.25;

pub struct MessageDrawable {
    /// Each line and where its baseline starts
    pub lines: Vec<(String, f32, f32)>,
    pub font_size: f32,
    pub color: Color,
    /// A rectangle to dim behind the text: x, y, width, height
    pub background: Option<(f32, f32, f32, f32)>,
}

impl MessageDrawable {
    /// Draws the message text. The caller must have set the **default (screen-
    /// space) camera** (`set_default_camera`) beforehand — text lives in
    /// screen space, not the board-offset camera.
    pub fn draw(&self) {
        if let Some((x, y, w, h)) = self.background {
            // fades out with the text
            let alpha = BACKGROUND_ALPHA * self.color.a;
            draw_rectangle(x, y, w, h, Color::new(0., 0., 0., alpha));
        }
        for (line, x, y) in &self.lines {
            draw_text(line, *x, *y, self.font_size, self.color);
        }
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
            let time_left = deadline.checked_duration_since(Instant::now())?;
            let millis = time_left.as_millis();
            if millis < 200 {
                color.a = millis as f32 / 200.;
            }
        }

        let size = self.font_size;
        let line_height = LINE_HEIGHT * size;
        // the first line's ascent, so the text's top sits at the margin
        let ascent = measure_text("X", size).offset_y;

        let mut width: f32 = 0.;
        let lines: Vec<_> = self
            .text
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let line_width = measure_text(line, size).width;
                width = width.max(line_width);
                let x = match self.position {
                    Position::TopLeft => self.h_margin,
                    Position::TopRight => screen_w - self.h_margin - line_width,
                };
                (line.to_string(), x, self.v_margin + ascent + i as f32 * line_height)
            })
            .collect();

        let background = self.background.then(|| {
            let padding = BACKGROUND_PADDING * size;
            // from the top of the first line to below the last one's descenders
            let height = ascent + (lines.len().max(1) - 1) as f32 * line_height + DESCENT * size;
            let x = match self.position {
                Position::TopLeft => self.h_margin,
                Position::TopRight => screen_w - self.h_margin - width,
            };
            (x - padding, self.v_margin - padding, width + 2. * padding, height + 2. * padding)
        });

        Some(MessageDrawable { lines, font_size: size, color, background })
    }
}
