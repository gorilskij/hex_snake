use crate::snake;
use ggez::graphics::Color;

macro_rules! gray {
    ($lightness:expr) => {
        Color {
            r: $lightness,
            g: $lightness,
            b: $lightness,
            a: 1.,
        }
    };
}

#[derive(Clone)]
pub struct Palette {
    pub grid_thickness: f32,
    pub grid_dot_radius: f32,
    pub border_thickness: f32,

    pub background_color: Color,
    pub grid_color: Color,
    pub grid_dot_color: Color,
    pub border_color: Color,
    pub apple_color: Color,

    pub palette_competitor: snake::PaletteTemplate,
    pub palette_killer: snake::PaletteTemplate,
    pub palette_rain: snake::PaletteTemplate,
}

#[allow(dead_code)]
impl Palette {
    pub fn dark() -> Self {
        Self {
            grid_thickness: 1.,
            grid_dot_radius: 2.,
            border_thickness: 3.,

            background_color: Color::BLACK,
            grid_color: gray!(0.25),
            grid_dot_color: Color::WHITE,
            border_color: Color::WHITE,
            apple_color: gray!(0.45),

            palette_competitor: snake::PaletteTemplate::pastel_rainbow(true),
            palette_killer: snake::PaletteTemplate::dark_blue_to_red(false),
            // palette_killer: snake::PaletteTemplate::dark_rainbow(true),
            palette_rain: snake::PaletteTemplate::gray_gradient(0.5, false),
        }
    }
}
