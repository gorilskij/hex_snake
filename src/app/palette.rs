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

/// The colors of border hints, by what the player would run into across a
/// wrap-around edge.
#[derive(Copy, Clone)]
pub struct HintColors {
    pub crash: Color,
    pub cut: Color,
    pub pass: Color,
    pub apple: Color,
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

    /// Hints recoloring stretches of the border
    pub border_hint_colors: HintColors,
    /// Gradient hints (the color at the edge, fading out into the cell)
    pub gradient_hint_colors: HintColors,

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

            border_hint_colors: HintColors {
                crash: Color::new(0.72, 0.16, 0.16, 1.),
                cut: Color::new(0.25, 0.4, 0.8, 1.),
                pass: Color::new(0.86, 0.72, 0.2, 1.),
                apple: Color::new(0.22, 0.6, 0.28, 1.),
            },
            gradient_hint_colors: HintColors {
                crash: Color::new(1., 0.1, 0.1, 0.4),
                cut: Color::new(0.2, 0.4, 1., 0.4),
                pass: Color::new(1., 0.85, 0.1, 0.4),
                apple: Color::new(0.2, 0.9, 0.3, 0.4),
            },

            palette_competitor: snake::PaletteTemplate::pastel_rainbow(),
            palette_killer: snake::PaletteTemplate::dark_blue_to_red(),
            // palette_killer: snake::PaletteTemplate::dark_rainbow(),
            palette_rain: snake::PaletteTemplate::gray_gradient(0.5),
        }
    }
}
