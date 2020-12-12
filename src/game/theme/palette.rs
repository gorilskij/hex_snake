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
    // (head, tail), the rest is shaded in
    pub snake_color: (Color, Color), // single player
    pub snake1_color: (Color, Color), // 2-player
    pub snake2_color: (Color, Color), // 2-player

    pub crash_color: Color,
    pub eaten_color: Color,
    pub teleported_color: Color,
}

#[allow(dead_code)]
impl Palette {
    pub fn light() -> Self {
        Self {
            background_color: WHITE,
            foreground_color: BLACK,
            apple_fill_color: Color::from_rgb(255, 0, 0),
            snake_color: (BLACK, gray!(0.5)),
            snake1_color: (
                Color::from_rgb(0, 192, 0),
                Color::from_rgb(0, 64, 0),
            ),
            snake2_color: (
                Color::from_rgb(16, 169, 224),
                Color::from_rgb(12, 129, 171),
            ),
            crash_color: Color::from_rgb(255, 0, 128),
            eaten_color: Color::from_rgb(0, 255, 128),
            teleported_color: Color::from_rgb(245, 192, 64),
        }
    }


    pub fn dark() -> Self {
        Self {
            background_color: BLACK,
            foreground_color: gray!(0.25),
            apple_fill_color: Color::from_rgb(255, 0, 0),
            snake_color: (gray!(0.75), gray!(0.25)),
            snake1_color: (
                Color::from_rgb(0, 192, 0),
                Color::from_rgb(0, 64, 0),
            ),
            snake2_color: (
                Color::from_rgb(16, 169, 224),
                Color::from_rgb(4, 52, 69),
            ),
            crash_color: Color::from_rgb(255, 0, 128),
            eaten_color: Color::from_rgb(0, 255, 128),
            teleported_color: Color::from_rgb(245, 192, 64),
        }
    }
}
