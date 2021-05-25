use ggez::graphics::Color;
use hsl::HSL;

use SegmentType::*;

use crate::{
    app::snake::{SegmentType, SnakeBody},
    oklab::OkLab,
};

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
pub enum PaletteTemplate {
    RGBGradient {
        head: Color,
        tail: Color,
        eaten: EatenColor,
        // keeps track of the longest length achieved so far and uses that as the tail color
        persistent: bool,
    },
    HSLGradient {
        head_hue: f64,
        tail_hue: f64,
        lightness: f64,
        eaten_lightness: f64,
        persistent: bool,
    },
    OkLabGradient {
        head_hue: f64,
        tail_hue: f64,
        lightness: f64,
        eaten_lightness: f64,
        persistent: bool,
    },
}

// TODO: write a builder
#[allow(dead_code)]
impl PaletteTemplate {
    pub fn rgb_gradient(head: Color, tail: Color, eaten: Option<Color>, persistent: bool) -> Self {
        Self::RGBGradient {
            head,
            tail,
            eaten: EatenColor::Fixed(eaten.unwrap_or(*DEFAULT_EATEN_COLOR)),
            persistent,
        }
    }

    pub fn gray_gradient(persistent: bool) -> Self {
        Self::rgb_gradient(gray!(0.72), gray!(0.25), None, persistent)
    }

    pub fn hsl_gradient(
        head_hue: f64,
        tail_hue: f64,
        lightness: f64,
        eaten_lightness: f64,
        persistent: bool,
    ) -> Self {
        Self::HSLGradient {
            head_hue,
            tail_hue,
            lightness,
            eaten_lightness,
            persistent,
        }
    }

    pub fn oklab_gradient(
        head_hue: f64,
        tail_hue: f64,
        lightness: f64,
        eaten_lightness: f64,
        persistent: bool,
    ) -> Self {
        Self::OkLabGradient {
            head_hue,
            tail_hue,
            lightness,
            eaten_lightness,
            persistent,
        }
    }

    // red -> purple
    const HSL_RAINBOW: (f64, f64) = (0., 273.);

    // green -> red (yellows are very ugly in oklab)
    const OKLAB_RAINBOW: (f64, f64) = (147.3, 428.);

    pub fn rainbow(persistent: bool) -> Self {
        Self::hsl_gradient(
            Self::HSL_RAINBOW.0,
            Self::HSL_RAINBOW.1,
            0.4,
            0.7,
            persistent,
        )
    }

    pub fn pastel_rainbow(persistent: bool) -> Self {
        Self::hsl_gradient(
            Self::HSL_RAINBOW.0,
            Self::HSL_RAINBOW.1,
            0.75,
            0.7,
            persistent,
        )
    }

    pub fn dark_rainbow(persistent: bool) -> Self {
        Self::hsl_gradient(
            Self::HSL_RAINBOW.0,
            Self::HSL_RAINBOW.1,
            0.2,
            0.2,
            persistent,
        )
    }

    pub fn green_to_red(persistent: bool) -> Self {
        Self::oklab_gradient(
            Self::OKLAB_RAINBOW.0,
            Self::OKLAB_RAINBOW.1,
            0.6,
            0.7,
            persistent,
        )
    }

    pub fn dark_blue_to_red(persistent: bool) -> Self {
        Self::oklab_gradient(250., Self::OKLAB_RAINBOW.1, 0.3, 0.3, persistent)
    }
}

#[derive(Copy, Clone)]
pub enum SegmentStyle {
    Solid(Color),
    RGBGradient(Color, Color),
    HSLGradient(Color, Color),
}

pub trait Palette {
    fn segment_styles(&mut self, body: &SnakeBody) -> Vec<SegmentStyle>;
}

impl From<PaletteTemplate> for Box<dyn Palette> {
    fn from(template: PaletteTemplate) -> Self {
        match template {
            PaletteTemplate::RGBGradient { head, tail, eaten, persistent } => {
                Box::new(RGBGradient {
                    head,
                    tail,
                    eaten,
                    max_len: persistent.then(|| 0),
                })
            }
            PaletteTemplate::HSLGradient {
                head_hue,
                tail_hue,
                lightness,
                eaten_lightness,
                persistent,
            } => Box::new(HSLGradient {
                head_hue,
                tail_hue,
                lightness,
                eaten_lightness,
                max_len: persistent.then(|| 0),
            }),
            PaletteTemplate::OkLabGradient {
                head_hue,
                tail_hue,
                lightness,
                eaten_lightness,
                persistent,
            } => Box::new(OkLabGradient {
                head_hue,
                tail_hue,
                lightness,
                eaten_lightness,
                max_len: persistent.then(|| 0),
            }),
        }
    }
}

// if max_len is None, use body.len(), otherwise, update max_len to be the maximum of itself and body.len() and use that
fn and_update_max_len(max_len: &mut Option<usize>, body_len: usize) -> usize {
    match max_len {
        Some(len) => {
            if body_len > *len {
                *len = body_len;
            }
            *len
        }
        None => body_len,
    }
}

pub struct RGBGradient {
    head: Color,
    tail: Color,
    eaten: EatenColor,
    max_len: Option<usize>,
}

impl Palette for RGBGradient {
    fn segment_styles(&mut self, body: &SnakeBody) -> Vec<SegmentStyle> {
        let mut styles = Vec::with_capacity(body.len());

        let len = and_update_max_len(&mut self.max_len, body.len());
        for (i, seg) in body.iter().enumerate() {
            let color = if seg.typ == Crashed {
                *DEFAULT_CRASHED_COLOR
            } else {
                let r = i + body.missing_front;
                let head_ratio = 1. - r as f32 / (len - 1) as f32;
                let tail_ratio = 1. - head_ratio;
                let normal_color = Color {
                    r: head_ratio * self.head.r + tail_ratio * self.tail.r,
                    g: head_ratio * self.head.g + tail_ratio * self.tail.g,
                    b: head_ratio * self.head.b + tail_ratio * self.tail.b,
                    a: 1.,
                };

                match seg.typ {
                    Normal | BlackHole => normal_color,
                    Eaten { .. } => self.eaten.paint_segment(&normal_color),
                    Crashed => *DEFAULT_CRASHED_COLOR,
                }
            };
            styles.push(SegmentStyle::Solid(color));
        }

        styles
    }
}

pub struct HSLGradient {
    head_hue: f64,
    tail_hue: f64,
    lightness: f64,
    eaten_lightness: f64,
    max_len: Option<usize>, // optional feature
}

impl Palette for HSLGradient {
    fn segment_styles(&mut self, body: &SnakeBody) -> Vec<SegmentStyle> {
        let mut styles = Vec::with_capacity(body.len());

        let len = and_update_max_len(&mut self.max_len, body.len());
        for (i, seg) in body.iter().enumerate() {
            if seg.typ == Crashed {
                styles.push(SegmentStyle::Solid(*DEFAULT_CRASHED_COLOR));
            } else {
                let r = i + body.missing_front;
                let start_hue = self.head_hue + (self.tail_hue - self.head_hue) * r as f64 / len as f64;
                let end_hue = self.head_hue + (self.tail_hue - self.head_hue) * (r + 1) as f64 / len as f64;
                let (start_color, end_color) = match seg.typ {
                    Normal | BlackHole => {
                        let start_hsl = HSL { h: start_hue, s: 1., l: self.lightness };
                        let end_hsl = HSL { h: end_hue, s: 1., l: self.lightness };
                        (Color::from(start_hsl.to_rgb()), Color::from(end_hsl.to_rgb()))
                    }
                    Eaten { .. } => {
                        // invert lightness twice
                        let start_hsl = HSL {
                            h: start_hue,
                            s: 1.,
                            l: 1. - self.eaten_lightness,
                        };
                        let end_hsl = HSL {
                            h: end_hue,
                            s: 1.,
                            l: 1. - self.eaten_lightness,
                        };
                        (
                            Color::from(invert_rgb(start_hsl.to_rgb())),
                            Color::from(invert_rgb(end_hsl.to_rgb())),
                        )
                    }
                    Crashed => unreachable!(),
                };
                // styles.push(SegmentStyle::HSLGradient(start_color, end_color));
                styles.push(SegmentStyle::Solid(start_color));
            }
        }

        styles
    }
}

pub struct OkLabGradient {
    head_hue: f64,
    tail_hue: f64,
    lightness: f64,
    eaten_lightness: f64,
    max_len: Option<usize>,
}

impl Palette for OkLabGradient {
    fn segment_styles(&mut self, body: &SnakeBody) -> Vec<SegmentStyle> {
        let mut styles = Vec::with_capacity(body.len());

        let len = and_update_max_len(&mut self.max_len, body.len());
        for (i, seg) in body.iter().enumerate() {
            let color = if seg.typ == Crashed {
                *DEFAULT_CRASHED_COLOR
            } else {
                let r = i + body.missing_front;
                let hue = self.head_hue + (self.tail_hue - self.head_hue) * r as f64 / len as f64;
                match seg.typ {
                    Normal | BlackHole => {
                        let oklab = OkLab::from_lch(self.lightness, 0.5, hue);
                        Color::from(oklab.to_rgb())
                    }
                    Eaten { .. } => {
                        // invert lightness twice
                        let oklab = OkLab::from_lch(1. - self.eaten_lightness, 0.5, hue);
                        Color::from(invert_rgb(oklab.to_rgb()))
                    }
                    Crashed => *DEFAULT_CRASHED_COLOR,
                }
            };
            styles.push(SegmentStyle::Solid(color));
        }

        styles
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
