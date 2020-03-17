mod palette;

pub use palette::Palette;

pub struct Theme {
    pub palette: Palette,
    pub line_thickness: f32,
}

#[allow(dead_code)]
impl Theme {
    pub const DEFAULT_LIGHT: Self = Self {
        palette: Palette::LIGHT,
        line_thickness: 2.,
    };

    pub const DEFAULT_DARK: Self = Self {
        palette: Palette::DARK,
        line_thickness: 1.,
    };
}