use macroquad::color::Color;

use crate::snake;

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

    /// Border hints (strongest, at the edge): what the player would run into
    /// across a wrap-around edge
    pub hint_crash_color: Color,
    pub hint_cut_color: Color,
    pub hint_pass_color: Color,
    pub hint_apple_color: Color,

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

            background_color: crate::color::BLACK,
            grid_color: gray!(0.25),
            grid_dot_color: crate::color::WHITE,
            border_color: crate::color::WHITE,
            apple_color: gray!(0.45),

            hint_crash_color: Color::new(1., 0.1, 0.1, 0.4),
            hint_cut_color: Color::new(0.2, 0.4, 1., 0.4),
            hint_pass_color: Color::new(1., 0.85, 0.1, 0.4),
            hint_apple_color: Color::new(0.2, 0.9, 0.3, 0.4),

            palette_competitor: snake::PaletteTemplate::pastel_rainbow(),
            palette_killer: snake::PaletteTemplate::dark_blue_to_red(),
            // palette_killer: snake::PaletteTemplate::dark_rainbow(),
            palette_rain: snake::PaletteTemplate::gray_gradient(0.5),
        }
    }
}
