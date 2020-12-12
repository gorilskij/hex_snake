mod palette;

pub use palette::Palette;

pub struct Theme {
    pub palette: Palette,
    pub line_thickness: f32,
}

// unify theme and palette?
#[allow(dead_code)]
impl Theme {
    pub fn default_light() -> Self {
        Self {
            palette: Palette::light(),
            line_thickness: 2.,
        }
    }

    pub fn default_dark() -> Self {
        Self {
            palette: Palette::dark(),
            line_thickness: 1.,
        }
    }
}
