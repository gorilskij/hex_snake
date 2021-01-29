use crate::{
    app::snake::{Segment, SegmentType},
    oklab::OkLab,
};
use ggez::graphics::Color;
use hsl::HSL;
use std::cmp::max;
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

lazy_static! {
    static ref DEFAULT_EATEN_COLOR: Color = Color::from_rgb(0, 255, 128);
    static ref DEFAULT_CRASHED_COLOR: Color = Color::from_rgb(255, 0, 128);
    static ref DEFAULT_BLACK_HOLE_COLOR: Color = Color::from_rgb(245, 197, 66);
    // static ref DEFAULT_PORTAL_COLOR: Color = Color::from_rgb(245, 192, 64);
}

#[derive(Clone)]
pub enum EatenColor {
    Fixed(Color),
    RGBInverted,
    // HSLInverted, // 180deg rotation?
}

fn invert_rgb(rgb: (u8, u8, u8)) -> (u8, u8, u8) {
    (255 - rgb.0, 255 - rgb.1, 255 - rgb.2)
}

impl EatenColor {
    fn paint_segment(&self, normal_color: &Color) -> Color {
        match self {
            EatenColor::Fixed(color) => *color,
            EatenColor::RGBInverted => {
                let (r, g, b) = normal_color.to_rgb();
                Color::from_rgb(255 - r, 255 - g, 255 - b)
            }
        }
    }
}

#[derive(Clone)]
pub enum SnakePaletteTemplate {
    RGBGradient {
        head: Color,
        tail: Color,
        eaten: EatenColor,
    },
    HSLGradient {
        head_hue: f64,
        tail_hue: f64,
        lightness: f64,
        eaten_lightness: f64,
    },
    OkLabGradient {
        head_hue: f64,
        tail_hue: f64,
        lightness: f64,
        eaten_lightness: f64,
    },
    // keeps track of the longest length achieved
    Persistent(Box<SnakePaletteTemplate>),
}

#[allow(dead_code)]
impl SnakePaletteTemplate {
    pub fn rgb_gradient(head: Color, tail: Color, eaten: Option<Color>) -> Self {
        Self::RGBGradient {
            head,
            tail,
            eaten: EatenColor::Fixed(eaten.unwrap_or(*DEFAULT_EATEN_COLOR)),
        }
    }

    pub fn gray_gradient() -> Self {
        Self::rgb_gradient(gray!(0.72), gray!(0.25), None)
    }

    pub fn hsl_gradient(
        head_hue: f64,
        tail_hue: f64,
        lightness: f64,
        eaten_lightness: f64,
    ) -> Self {
        Self::HSLGradient {
            head_hue,
            tail_hue,
            lightness,
            eaten_lightness,
        }
    }

    pub fn oklab_gradient(
        head_hue: f64,
        tail_hue: f64,
        lightness: f64,
        eaten_lightness: f64,
    ) -> Self {
        Self::OkLabGradient {
            head_hue,
            tail_hue,
            lightness,
            eaten_lightness,
        }
    }

    // red -> purple
    const HSL_RAINBOW: (f64, f64) = (0., 273.);

    // green -> red (yellows are very ugly in oklab)
    const OKLAB_RAINBOW: (f64, f64) = (147.3, 428.);

    pub fn persistent(self) -> Self {
        match self {
            Self::Persistent(_) => self,
            template => Self::Persistent(Box::new(template)),
        }
    }

    pub fn rainbow() -> Self {
        Self::hsl_gradient(Self::HSL_RAINBOW.0, Self::HSL_RAINBOW.1, 0.4, 0.7)
    }

    pub fn pastel_rainbow() -> Self {
        Self::hsl_gradient(Self::HSL_RAINBOW.0, Self::HSL_RAINBOW.1, 0.75, 0.7)
    }

    pub fn dark_rainbow() -> Self {
        Self::hsl_gradient(Self::HSL_RAINBOW.0, Self::HSL_RAINBOW.1, 0.2, 0.2)
    }

    pub fn green_to_red() -> Self {
        Self::oklab_gradient(Self::OKLAB_RAINBOW.0, Self::OKLAB_RAINBOW.1, 0.6, 0.7)
    }

    pub fn dark_blue_to_red() -> Self {
        Self::oklab_gradient(250., Self::OKLAB_RAINBOW.1, 0.3, 0.3)
    }
}

pub trait SnakePainter {
    fn paint_segment(&mut self, seg_idx: usize, len: usize, segment: &Segment) -> Color;
}

impl From<SnakePaletteTemplate> for Box<dyn SnakePainter> {
    fn from(template: SnakePaletteTemplate) -> Self {
        match template {
            SnakePaletteTemplate::RGBGradient { head, tail, eaten } => {
                Box::new(RGBGradient { head, tail, eaten })
            }
            SnakePaletteTemplate::HSLGradient {
                head_hue,
                tail_hue,
                lightness,
                eaten_lightness,
            } => Box::new(HSLGradient {
                head_hue,
                tail_hue,
                lightness,
                eaten_lightness,
            }),
            SnakePaletteTemplate::OkLabGradient {
                head_hue,
                tail_hue,
                lightness,
                eaten_lightness,
            } => Box::new(OkLabGradient {
                head_hue,
                tail_hue,
                lightness,
                eaten_lightness,
            }),
            SnakePaletteTemplate::Persistent(palette) => Box::new(Persistent {
                painter: (*palette).into(),
                max_len: 0,
            }),
        }
    }
}

pub struct RGBGradient {
    head: Color,
    tail: Color,
    eaten: EatenColor,
}

impl SnakePainter for RGBGradient {
    fn paint_segment(&mut self, seg_idx: usize, len: usize, segment: &Segment) -> Color {
        if segment.typ == SegmentType::Crashed {
            return *DEFAULT_CRASHED_COLOR;
        }

        let head_ratio = 1. - seg_idx as f32 / (len - 1) as f32;
        let tail_ratio = 1. - head_ratio;
        let normal_color = Color {
            r: head_ratio * self.head.r + tail_ratio * self.tail.r,
            g: head_ratio * self.head.g + tail_ratio * self.tail.g,
            b: head_ratio * self.head.b + tail_ratio * self.tail.b,
            a: 1.,
        };

        match segment.typ {
            SegmentType::Normal => normal_color,
            SegmentType::Eaten { .. } => self.eaten.paint_segment(&normal_color),
            SegmentType::Crashed => *DEFAULT_CRASHED_COLOR,
            SegmentType::BlackHole => *DEFAULT_BLACK_HOLE_COLOR,
        }
    }
}

pub struct HSLGradient {
    head_hue: f64,
    tail_hue: f64,
    lightness: f64,
    eaten_lightness: f64,
}

impl SnakePainter for HSLGradient {
    fn paint_segment(&mut self, seg_idx: usize, len: usize, segment: &Segment) -> Color {
        if segment.typ == SegmentType::Crashed {
            return *DEFAULT_CRASHED_COLOR;
        }

        let hue = self.head_hue + (self.tail_hue - self.head_hue) * seg_idx as f64 / len as f64;
        match segment.typ {
            SegmentType::Normal => {
                let hsl = HSL {
                    h: hue,
                    s: 1.,
                    l: self.lightness,
                };
                Color::from(hsl.to_rgb())
            }
            SegmentType::Eaten { .. } => {
                // invert lightness twice
                let hsl = HSL {
                    h: hue,
                    s: 1.,
                    l: 1. - self.eaten_lightness,
                };
                Color::from(invert_rgb(hsl.to_rgb()))
            }
            SegmentType::Crashed => *DEFAULT_CRASHED_COLOR,
            SegmentType::BlackHole => *DEFAULT_BLACK_HOLE_COLOR,
        }
    }
}

pub struct OkLabGradient {
    head_hue: f64,
    tail_hue: f64,
    lightness: f64,
    eaten_lightness: f64,
}

impl SnakePainter for OkLabGradient {
    fn paint_segment(&mut self, seg_idx: usize, len: usize, segment: &Segment) -> Color {
        if segment.typ == SegmentType::Crashed {
            return *DEFAULT_CRASHED_COLOR;
        }

        let hue = self.head_hue + (self.tail_hue - self.head_hue) * seg_idx as f64 / len as f64;
        match segment.typ {
            SegmentType::Normal => {
                let oklab = OkLab::from_lch(self.lightness, 0.5, hue);
                Color::from(oklab.to_rgb())
            }
            SegmentType::Eaten { .. } => {
                // invert lightness twice
                let oklab = OkLab::from_lch(1. - self.eaten_lightness, 0.5, hue);
                Color::from(invert_rgb(oklab.to_rgb()))
            }
            SegmentType::Crashed => *DEFAULT_CRASHED_COLOR,
            SegmentType::BlackHole => *DEFAULT_BLACK_HOLE_COLOR,
        }
    }
}

pub struct Persistent {
    painter: Box<dyn SnakePainter>,
    max_len: usize,
}

impl SnakePainter for Persistent {
    fn paint_segment(&mut self, seg_idx: usize, len: usize, segment: &Segment) -> Color {
        self.max_len = max(len, self.max_len);
        self.painter.paint_segment(seg_idx, self.max_len, segment)
    }
}

// old designs
// #[allow(dead_code)]
// impl SnakePalette {
//     pub fn checker(on_step: usize, off_step: usize) -> Self {
//         Self {
//             segment_color: Box::new(move |seg, len| {
//                 if seg % (on_step + off_step) < on_step || seg == len - 1 {
//                     WHITE
//                 } else {
//                     BLACK
//                 }
//             }),
//             eaten_color: Box::new(*DEFAULT_EATEN_COLOR),
//             crashed_color: *DEFAULT_CRASHED_COLOR,
//             portal_color: *DEFAULT_PORTAL_COLOR,
//         }
//     }
//
//     pub fn sin(period: usize) -> Self {
//         Self {
//             segment_color: Box::new(move |seg, _| {
//                 let x = seg as f32 * 2. * PI / period as f32;
//                 let l = (x.sin() + 1.) / 2.;
//                 Color {
//                     r: l,
//                     g: l,
//                     b: l,
//                     a: 1.,
//                 }
//             }),
//             eaten_color: Box::new(*DEFAULT_EATEN_COLOR),
//             crashed_color: *DEFAULT_CRASHED_COLOR,
//             portal_color: *DEFAULT_PORTAL_COLOR,
//         }
//     }
//
//     pub fn rainbow_sin(period: usize) -> Self {
//         Self {
//             segment_color: Box::new(move |seg, _| {
//                 let x = seg as f32 * 2. * PI / period as f32;
//                 let l = (x.sin() + 1.) / 2.;
//                 let h = (x / (2. * PI)).floor() * 30.;
//                 let hsl = HSL {
//                     h: h as f64 % 360.,
//                     s: 1.,
//                     l: l as f64 / 2.,
//                 };
//                 Color::from(hsl.to_rgb())
//             }),
//             eaten_color: Box::new(*DEFAULT_EATEN_COLOR),
//             crashed_color: *DEFAULT_CRASHED_COLOR,
//             portal_color: *DEFAULT_PORTAL_COLOR,
//         }
//     }
// }
