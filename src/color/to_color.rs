use super::Color;
use crate::color::oklab::OkLab;
use ggez::graphics;
use hsl::HSL;

pub trait ToColor {
    fn to_color(self) -> Color;
}

impl ToColor for HSL {
    fn to_color(self) -> Color {
        Color(graphics::Color::from(self.to_rgb()))
    }
}

impl ToColor for OkLab {
    fn to_color(self) -> Color {
        Color(graphics::Color::from(self.to_rgb()))
    }
}
