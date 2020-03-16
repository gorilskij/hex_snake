use ggez::graphics::{Color, WHITE, BLACK};

pub struct Palette {
    pub background_color: Color,
    pub foreground_color: Color,
    pub apple_fill_color: Color,
}

impl Palette {
    pub fn light() -> Self {
        Self {
            background_color: WHITE,
            foreground_color: BLACK,
            apple_fill_color: Color { r: 1., g: 0., b: 0., a: 1. },
        }
    }

    pub fn dark() -> Self {
        Self {
            background_color: BLACK,
            foreground_color: WHITE,
            apple_fill_color: Color { r: 1., g: 0., b: 0., a: 1. },
        }
    }
}