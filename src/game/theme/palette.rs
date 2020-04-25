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
    pub snake_head_color: Color,
    pub snake_tail_color: Color,
    pub snake_crash_color: Color,
}

#[allow(dead_code)]
impl Palette {
    pub const LIGHT: Self = Self {
        background_color: WHITE,
        foreground_color: BLACK,
        apple_fill_color: Color {
            r: 1.,
            g: 0.,
            b: 0.,
            a: 1.,
        },
        snake_head_color: BLACK,
        snake_tail_color: gray!(0.5),
        snake_crash_color: Color {
            r: 1.,
            b: 0.5,
            g: 0.,
            a: 1.,
        },
    };

    pub const DARK: Self = Self {
        background_color: BLACK,
        foreground_color: gray!(0.25),
        apple_fill_color: Color {
            r: 1.,
            g: 0.,
            b: 0.,
            a: 1.,
        },
        snake_head_color: gray!(0.75),
        snake_tail_color: gray!(0.25),
        snake_crash_color: Color {
            r: 1.,
            b: 0.5,
            g: 0.,
            a: 1.,
        },
    };
}
