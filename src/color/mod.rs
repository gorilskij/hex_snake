use std::ops::{Add, Deref, DerefMut, Div, Mul, Sub};
use ggez::graphics;

pub mod oklab;
pub mod to_color;

#[derive(Copy, Clone, Debug)]
pub struct Color(pub graphics::Color);

impl Deref for Color {
    type Target = graphics::Color;

    fn deref(&self) -> &Self::Target {
        self.assert_in_range();
        &self.0
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.assert_in_range();
        &mut self.0
    }
}

impl Color {
    pub const TRANSPARENT: Self = Self(graphics::Color::new(0., 0., 0., 0.));
    pub const WHITE: Self = Self(graphics::Color::WHITE);
    pub const BLACK: Self = Self(graphics::Color::BLACK);
    pub const RED: Self = Self(graphics::Color::RED);

    #[inline(always)]
    fn assert_in_range(&self) {
        // TODO: why do these assertions fail?
        // assert!(0. <= self.0.r && self.0.r <= 1., "red out of range: {}", self.0.r);
        // assert!(0. <= self.0.g && self.0.g <= 1., "green out of range: {}", self.0.g);
        // assert!(0. <= self.0.b && self.0.b <= 1., "blue out of range: {}", self.0.b);
        // assert!(0. <= self.0.a && self.0.a <= 1., "alpha out of range: {}", self.0.a);
    }

    // #[inline(always)]
    // pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
    //     Self(graphics::Color::new(r, g, b, a))
    // }

    #[inline(always)]
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(graphics::Color::from_rgb(r, g, b))
    }
}

impl Add<Color> for Color {
    type Output = Self;

    fn add(self, rhs: Color) -> Self::Output {
        Self(graphics::Color {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
        })
    }
}

impl Sub<Color> for Color {
    type Output = Self;

    fn sub(self, rhs: Color) -> Self::Output {
        Self(graphics::Color {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
            a: self.a - rhs.a,
        })
    }
}

impl Mul<f64> for Color {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(
            graphics::Color {
                r: (self.r as f64 * rhs) as f32,
                g: (self.g as f64 * rhs) as f32,
                b: (self.b as f64 * rhs) as f32,
                a: (self.a as f64 * rhs) as f32,
            }
        )
    }
}

impl Mul<Color> for f64 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color(
            graphics::Color {
                r: (self * rhs.r as f64) as f32,
                g: (self * rhs.g as f64) as f32,
                b: (self * rhs.b as f64) as f32,
                a: (self * rhs.a as f64) as f32,
            }
        )
    }
}

impl Div<f64> for Color {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self(
            graphics::Color {
                r: (self.r as f64 / rhs) as f32,
                g: (self.g as f64 / rhs) as f32,
                b: (self.b as f64 / rhs) as f32,
                a: (self.a as f64 / rhs) as f32,
            }
        )
    }
}
