use std::ops::{Add, Div, Mul, Sub};
use ggez::graphics;

pub mod oklab;
pub mod to_color;

#[derive(Deref, DerefMut, Copy, Clone, Debug)]
pub struct Color(pub graphics::Color);

impl Color {
    pub const TRANSPARENT: Self = Self(graphics::Color::new(0., 0., 0., 0.));
    pub const WHITE: Self = Self(graphics::Color::WHITE);
    pub const BLACK: Self = Self(graphics::Color::BLACK);
    pub const RED: Self = Self(graphics::Color::RED);

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
        assert!((self.a - rhs.a).abs() < f32::EPSILON);
        Self {
            0: graphics::Color {
                r: self.r + rhs.r,
                g: self.g + rhs.g,
                b: self.b + rhs.b,
                a: self.a,
            },
        }
    }
}

impl Sub<Color> for Color {
    type Output = Self;

    fn sub(self, rhs: Color) -> Self::Output {
        assert!((self.a - rhs.a).abs() < f32::EPSILON);
        Self {
            0: graphics::Color {
                r: self.r - rhs.r,
                g: self.g - rhs.g,
                b: self.b - rhs.b,
                a: self.a,
            },
        }
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
                a: self.a,
            }
        )
    }
}

impl Mul<Color> for f64 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color(
            graphics::Color {
                r: self as f32 * rhs.r,
                g: self as f32 * rhs.g,
                b: self as f32 * rhs.b,
                a: rhs.a,
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
                a: self.a,
            }
        )
    }
}
