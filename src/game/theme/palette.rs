use ggez::graphics::{Color, BLACK, WHITE};
use std::ops::Deref;
use hsl::HSL;

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

type SnakePaletteClosure = dyn Fn(usize, usize) -> Color;
pub struct SnakePalette(Box<SnakePaletteClosure>);

impl SnakePalette {
    fn gradient(head: Color, tail: Color) -> Self {
        Self(Box::new(move |seg, len| {
            let head_ratio = 1. - seg as f32 / (len - 1) as f32;
            let tail_ratio = 1. - head_ratio;
            Color {
                r: head_ratio * head.r + tail_ratio * tail.r,
                g: head_ratio * head.g + tail_ratio * tail.g,
                b: head_ratio * head.b + tail_ratio * tail.b,
                a: 1.,
            }
        }))
    }

    fn rainbow() -> Self {
        Self(Box::new(|seg, len| {
            let hue = 273. * seg as f64 / len as f64;
            let hsl = HSL { h: hue, s: 1., l: 0.4 };
            Color::from(hsl.to_rgb())
        }))
    }
}

impl Deref for SnakePalette {
    type Target = SnakePaletteClosure;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub struct Palette {
    pub background_color: Color, // cell color
    pub foreground_color: Color, // line color
    pub apple_fill_color: Color,

    pub single_player_snake: SnakePalette,
    pub player1_snake: SnakePalette,
    pub player2_snake: SnakePalette,

    pub crash_color: Color,
    pub eaten_color: Color,
    pub teleported_color: Color,
}

#[allow(dead_code)]
impl Palette {
    pub fn light() -> Self {
        todo!()
        // Self {
        //     background_color: WHITE,
        //     foreground_color: BLACK,
        //     apple_fill_color: Color::from_rgb(255, 0, 0),
        //     snake_color: (BLACK, gray!(0.5)),
        //     snake1_color: (
        //         Color::from_rgb(0, 192, 0),
        //         Color::from_rgb(0, 64, 0),
        //     ),
        //     snake2_color: (
        //         Color::from_rgb(16, 169, 224),
        //         Color::from_rgb(12, 129, 171),
        //     ),
        //     crash_color: Color::from_rgb(255, 0, 128),
        //     eaten_color: Color::from_rgb(0, 255, 128),
        //     teleported_color: Color::from_rgb(245, 192, 64),
        // }
    }


    pub fn dark() -> Self {
        Self {
            background_color: BLACK,
            foreground_color: gray!(0.25),
            apple_fill_color: Color::from_rgb(255, 0, 0),
            // single_player_snake: SnakePalette::gradient(gray!(0.75), gray!(0.25)),
            single_player_snake: SnakePalette::rainbow(),
            player1_snake: SnakePalette::gradient(
                Color::from_rgb(0, 192, 0),
                Color::from_rgb(0, 64, 0),
            ),
            // player2_snake: SnakePalette::gradient(
            //     Color::from_rgb(16, 169, 224),
            //     Color::from_rgb(4, 52, 69),
            // ),
            player2_snake: SnakePalette::rainbow(),
            // tropical version
            // player2_snake: SnakePalette::gradient(
            //     Color::from_rgb(16, 169, 224),
            //     Color::from_rgb(227, 173, 11),
            // ),
            crash_color: Color::from_rgb(255, 0, 128),
            eaten_color: Color::from_rgb(0, 255, 128),
            teleported_color: Color::from_rgb(245, 192, 64),
        }
    }
}
