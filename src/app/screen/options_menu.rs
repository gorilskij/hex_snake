//! The in-game options menu: a dimmed overlay over the paused game with a
//! centered column of wide hexagon buttons, one per option.
//!
//! Immediate-mode: the buttons are rebuilt from their labels every frame, so
//! they always show the current values and follow the window size. A column
//! that doesn't fit the window scrolls.

use macroquad::color::Color;
use macroquad::input::mouse_wheel;
use macroquad::shapes::draw_rectangle;
use macroquad::window::{screen_height, screen_width};

use crate::basic::Point;
use crate::button::style::{BUTTON_CELL_DIM, BUTTON_COLOR, FONT_SIZE, STROKE_THICKNESS};
use crate::button::{Button, ButtonData};
use crate::rendering::shape::WideHexagon;
use crate::support::text::{draw_text, measure_text};

/// Opacity of the black overlay between the game and the menu
const OVERLAY_ALPHA: f32 = 0.9;
/// Share of the window's width each button takes
const BUTTON_WIDTH: f32 = 0.75;
/// Space between lines, and the extra space of a [`Line::Gap`], relative to a
/// button's height
const GAP: f32 = 0.25;
const SECTION_GAP: f32 = 0.5;

/// Dim the whole window, for a menu to be drawn over the game.
pub fn draw_overlay() {
    draw_rectangle(0., 0., screen_width(), screen_height(), Color::new(0., 0., 0., OVERLAY_ALPHA));
}

/// A full-size wide hexagon button `width` wide
pub fn wide_button(width: f32, text: &str) -> ButtonData {
    let h_side = (width - 2. * BUTTON_CELL_DIM.cos).max(0.);
    ButtonData::new(WideHexagon::with_h_side(BUTTON_CELL_DIM, h_side), STROKE_THICKNESS, BUTTON_COLOR).text(
        text,
        FONT_SIZE,
        BUTTON_COLOR,
    )
}

/// An "are you sure?" screen: `question` over Yes and No. Returns the answer
/// once one is clicked.
pub fn draw_confirm(question: &str) -> Option<bool> {
    draw_overlay();
    let (width, height) = (screen_width(), screen_height());

    let dims = measure_text(question, FONT_SIZE);
    let button_height = BUTTON_CELL_DIM.height();
    let text_y = height / 2. - button_height;
    draw_text(question, (width - dims.width) / 2., text_y, FONT_SIZE, BUTTON_COLOR.normal);

    let button_width = BUTTON_CELL_DIM.side * 5.;
    let gap = BUTTON_CELL_DIM.side * 2.;
    let left = (width - 2. * button_width - gap) / 2.;
    let y = text_y + button_height / 2.;

    let mut answer = None;
    for (i, (text, value)) in [("Yes", true), ("No", false)].into_iter().enumerate() {
        let x = left + i as f32 * (button_width + gap);
        if Button::click(Point { x, y }, wide_button(button_width, text)).draw() {
            answer = Some(value);
        }
    }
    answer
}

/// A line of the options menu
pub enum Line {
    /// Buttons side by side, sharing the line's width
    Buttons(Vec<String>),
    /// Extra space
    Gap,
}

/// Draw the overlay and the lines, top to bottom, and return which button was
/// clicked this frame, if any, numbering the buttons in reading order. The
/// caller must have set the default (screen-space) camera.
///
/// A column taller than the window scrolls with the mouse wheel; `scroll` is
/// how far down it is, in pixels, kept between frames by the caller.
pub fn draw(lines: &[Line], scroll: &mut f32) -> Option<usize> {
    let (width, height) = (screen_width(), screen_height());
    draw_overlay();

    let margin = BUTTON_CELL_DIM.side;
    let button_height = BUTTON_CELL_DIM.height();
    let line_height = |line: &Line| match line {
        Line::Buttons(_) => button_height,
        Line::Gap => SECTION_GAP * button_height,
    };
    let gaps = lines.len().saturating_sub(1) as f32 * GAP * button_height;
    let total_height = lines.iter().map(line_height).sum::<f32>() + gaps;

    // the wheel reports how far the content should move: up is positive
    let overflow = (total_height - (height - 2. * margin)).max(0.);
    *scroll = (*scroll - mouse_wheel().1).clamp(0., overflow);
    let mut y = if overflow > 0. {
        margin - *scroll
    } else {
        (height - total_height) / 2.
    };

    let line_width = BUTTON_WIDTH * width;
    let left = (width - line_width) / 2.;

    let mut index = 0;
    let mut clicked = None;
    for line in lines {
        if let Line::Buttons(labels) = line {
            let n = labels.len() as f32;
            let button_width = (line_width - (n - 1.) * margin) / n;
            for (i, label) in labels.iter().enumerate() {
                let x = left + i as f32 * (button_width + margin);
                if Button::click(Point { x, y }, wide_button(button_width, label)).draw() {
                    clicked = Some(index);
                }
                index += 1;
            }
        }
        y += line_height(line) + GAP * button_height;
    }
    clicked
}
