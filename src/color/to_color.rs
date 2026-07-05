use hsl::HSL;
use macroquad::color::Color;

use crate::color::oklab::OkLab;

pub trait ToColor {
    fn to_color(self) -> Color;
}

impl ToColor for HSL {
    fn to_color(self) -> Color {
        let (r, g, b) = self.to_rgb();
        Color::from_rgba(r, g, b, 255)
    }
}

impl ToColor for OkLab {
    fn to_color(self) -> Color {
        let (r, g, b) = self.to_rgb();
        Color::from_rgba(r, g, b, 255)
    }
}
