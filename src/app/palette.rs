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
