use hsl::HSL;
use macroquad::color::Color;

use crate::color::to_color::ToColor;
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
    pub bad_apple: Color,
}

/// The teleport hint's color as the head closes in on a wrap: a sweep through
/// HSL hue, from `far_hue` while the wrap is merely in range to `near_hue` as
/// the head arrives, in step with the exit triangle opening.
///
/// The hue is interpolated in the direction written, so 280 -> 0 goes *down*
/// through blue, green and yellow rather than the short way round via magenta.
#[derive(Copy, Clone)]
pub struct TeleportHintColors {
    pub far_hue: f64,
    pub near_hue: f64,
    pub saturation: f64,
    pub lightness: f64,
}

impl TeleportHintColors {
    /// The color at `progress` along the sweep, 0 at the far end and 1 at the
    /// near one.
    pub fn at(self, progress: f32) -> Color {
        HSL {
            h: self.far_hue + (self.near_hue - self.far_hue) * progress as f64,
            s: self.saturation,
            l: self.lightness,
        }
        .to_color()
    }
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
    pub bad_apple_color: Color,

    /// Hints recoloring stretches of the border
    pub border_hint_colors: HintColors,
    /// Gradient hints (the color at the edge, fading out into the cell)
    pub gradient_hint_colors: HintColors,
    /// Teleport hints, painted over the border stretches a wrap leaves through
    /// and arrives at, sweeping from purple to red as the head closes in
    pub teleport_hint_colors: TeleportHintColors,
    /// Line hints, drawn over the snake from under its head out to the borders
    pub hint_line_color: Color,
    pub hint_line_thickness: f32,

    pub palette_competitor: snake::PaletteTemplate,
    pub palette_killer: snake::PaletteTemplate,
    pub palette_rain: snake::PaletteTemplate,
}

#[allow(dead_code)]
impl Palette {
    pub fn dark() -> Self {
        let hint_red = Color::new(0.72, 0.16, 0.16, 1.);
        // vomit green
        let bad_apple = Color::new(0.55, 0.62, 0.1, 1.);

        Self {
            grid_thickness: 1.,
            grid_dot_radius: 2.,
            border_thickness: 3.,

            background_color: crate::color::BLACK,
            grid_color: gray!(0.25),
            grid_dot_color: crate::color::WHITE,
            border_color: crate::color::WHITE,
            apple_color: gray!(0.45),
            bad_apple_color: bad_apple,

            border_hint_colors: HintColors {
                crash: hint_red,
                cut: Color::new(0.25, 0.4, 0.8, 1.),
                pass: Color::new(0.86, 0.72, 0.2, 1.),
                apple: Color::new(0.22, 0.6, 0.28, 1.),
                bad_apple,
            },
            gradient_hint_colors: HintColors {
                crash: Color::new(1., 0.1, 0.1, 0.4),
                cut: Color::new(0.2, 0.4, 1., 0.4),
                pass: Color::new(1., 0.85, 0.1, 0.4),
                apple: Color::new(0.2, 0.9, 0.3, 0.4),
                bad_apple: bad_apple.with_alpha(0.4),
            },
            // ends on the border hints' red, having started at purple
            teleport_hint_colors: TeleportHintColors {
                far_hue: 280.,
                near_hue: 0.,
                saturation: 0.636,
                lightness: 0.44,
            },
            hint_line_color: Color::new(0.5, 0.5, 0.5, 0.5),
            hint_line_thickness: 10.,

            palette_competitor: snake::PaletteTemplate::pastel_rainbow(),
            palette_killer: snake::PaletteTemplate::dark_blue_to_red(),
            // palette_killer: snake::PaletteTemplate::dark_rainbow(),
            palette_rain: snake::PaletteTemplate::gray_gradient(0.5),
        }
    }
}
