use ggez::graphics::{Color, BLACK, WHITE};

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

pub struct Palette {
    pub background_color: Color, // cell color
    pub foreground_color: Color, // line color
    pub apple_fill_color: Color,
    pub normal_color: (Color, Color), // (head, tail), the rest is shaded in
    pub crash_color: Color,
    pub eaten_color: Color,
    pub teleported_color: Color,
}

#[allow(dead_code)]
impl Palette {
    pub const LIGHT: Self = Self {
        background_color: WHITE,
        foreground_color: BLACK,
        apple_fill_color: Color { r: 1., g: 0., b: 0., a: 1., },
        normal_color: (BLACK, gray!(0.5)),
        crash_color: Color { r: 1., b: 0.5, g: 0., a: 1. },
        eaten_color: Color { r: 0., g: 1., b: 0.5, a: 1. },
        teleported_color: Color { r: 0.96, g: 0.75, b: 0.26, a: 1. },
    };

    pub const DARK: Self = Self {
        background_color: BLACK,
        foreground_color: gray!(0.25),
        apple_fill_color: Color { r: 1., g: 0., b: 0., a: 1. },
        normal_color: (gray!(0.75), gray!(0.25)),
        crash_color: Color { r: 1., b: 0.5, g: 0., a: 1. },
        eaten_color: Color { r: 0., g: 1., b: 0.5, a: 1. },
        teleported_color: Color { r: 0.96, g: 0.75, b: 0.26, a: 1. },
    };
}
