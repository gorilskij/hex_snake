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

pub struct Palette {
    pub grid_thickness: f32,
    pub grid_dot_radius: f32,
    pub border_thickness: f32,

    pub background_color: Color,
    pub grid_color: Color,
    pub grid_dot_color: Color,
    pub border_color: Color,
    pub apple_color: Color,
}

#[allow(dead_code)]
impl Palette {
    pub fn light() -> Self {
        todo!()
        // Self {
        //     line_thickness: 2.,
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
            grid_thickness: 1.,
            grid_dot_radius: 2.,
            border_thickness: 3.,

            background_color: Color::BLACK,
            grid_color: gray!(0.25),
            grid_dot_color: Color::WHITE,
            border_color: Color::WHITE,
            apple_color: gray!(0.45),
        }
    }
}
