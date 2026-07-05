//! Color utilities on top of `macroquad::color::Color` (used directly as the
//! color type everywhere). This module only adds things macroquad doesn't:
//! perceptual color spaces (`oklab`), conversions into a color (`to_color`), and
//! a `lerp` helper (macroquad's `Color` has no arithmetic operators, so blends go
//! through its `Vec4` form).

use macroquad::color::Color;

pub mod oklab;
pub mod to_color;

// Pure named colors. macroquad ships raylib's palette (e.g. its `RED` is
// (0.9, 0.16, 0.22)); the game wants pure primaries, so define our own.
pub const WHITE: Color = Color::new(1., 1., 1., 1.);
pub const BLACK: Color = Color::new(0., 0., 0., 1.);
pub const RED: Color = Color::new(1., 0., 0., 1.);
pub const GREEN: Color = Color::new(0., 1., 0., 1.);
pub const BLUE: Color = Color::new(0., 0., 1., 1.);
pub const YELLOW: Color = Color::new(1., 1., 0., 1.);
pub const MAGENTA: Color = Color::new(1., 0., 1., 1.);
pub const TRANSPARENT: Color = Color::new(0., 0., 0., 0.);

pub fn lerp(a: Color, b: Color, t: f32) -> Color {
    Color::from_vec(a.to_vec() * (1.0 - t) + b.to_vec() * t)
}
