use ggez::graphics::Color;
use hsl::HSL;

use SegmentType::*;

use crate::{
    app::snake::{SegmentType, SnakeBody},
    basic::HexPoint,
    color::oklab::OkLab,
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
    Alternating {
        color1: Color,
        color2: Color,
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
    const HSL_RAINBOW: (f64, f64) = (-20., 290.);

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

    pub fn alternating_white() -> Self {
        Self::Alternating {
            color1: Color::WHITE,
            color2: Color::new(0., 0., 0., 0.),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SegmentStyle {
    Solid(Color),
    RGBGradient {
        start_rgb: (u8, u8, u8),
        end_rgb: (u8, u8, u8),
    },
    HSLGradient {
        start_hue: f64,
        end_hue: f64,
        lightness: f64,
    },
}

impl SegmentStyle {
    pub fn into_solid(self) -> Self {
        match self {
            Self::Solid(_) => self,
            Self::RGBGradient { start_rgb, .. } => Self::Solid(Color::from(start_rgb)),
            Self::HSLGradient { start_hue, lightness, .. } => Self::Solid(Color::from(
                HSL { h: start_hue, s: 1., l: lightness }.to_rgb(),
            )),
        }
    }
}

pub trait Palette {
    fn segment_styles(&mut self, body: &SnakeBody, frame_frac: f32) -> Vec<SegmentStyle>;
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
            PaletteTemplate::Alternating { color1, color2 } => Box::new(Alternating {
                color1,
                color2,
                iteration: true,
                last_head: None,
            }),
        }
    }
}

// if max_len is None, use body.len(), otherwise, update max_len
//  to be the maximum of itself and body.len() and use that,
//  this is used to implement persistent rainbows
// TODO: implement variable resolution asking the palette how
//  much it needs, persistent rainbows don't need high
//  resolutions even in short snakes
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

/// Correct an integer snake length to an f64 length
/// that accounts for fractional segments, eaten segments
/// and growing
fn correct_len(len: usize, body: &SnakeBody, frame_frac: f64) -> f64 {
    let len = len as f64;
    if let SegmentType::Eaten { original_food, food_left } = body[body.len() - 1].typ
    {
        // Correct for eaten segment at the tail and
        //  fractional segment at the head (the eaten
        //  segment reduces in size more slowly than
        //  the head segment grows)

        // The actual visual length of the eaten segment
        //  at the tail of the snake
        let eaten_segment_frac =
            (food_left as f64 + 1. - frame_frac as f64) / (original_food + 1) as f64;

        len - 1. + eaten_segment_frac + frame_frac
    } else if body.grow > 0 {
        // If growth is happening for a reason other
        //  than eating (such as at the beginning of
        //  the game), correct only for the head
        len + frame_frac as f64
    } else {
        // If the snake isn't growing, the head and
        //  tail corrections cancel out
        len
    }
}

pub struct RGBGradient {
    head: Color,
    tail: Color,
    eaten: EatenColor,
    max_len: Option<usize>,
}

impl Palette for RGBGradient {
    fn segment_styles(&mut self, body: &SnakeBody, frame_frac: f32) -> Vec<SegmentStyle> {
        let mut styles = Vec::with_capacity(body.len());

        let len = and_update_max_len(&mut self.max_len, body.len());
        let len = correct_len(len, body, frame_frac as f64) as f32;
        for (i, seg) in body.iter().enumerate() {
            let color = if seg.typ == Crashed {
                *DEFAULT_CRASHED_COLOR
            } else {
                let r = (i + body.missing_front) as f32 + frame_frac;
                let head_ratio = 1. - r / (len - 1.);
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
    fn segment_styles(&mut self, body: &SnakeBody, frame_frac: f32) -> Vec<SegmentStyle> {
        let mut styles = Vec::with_capacity(body.len());

        let len = and_update_max_len(&mut self.max_len, body.len());
        let len = correct_len(len, body, frame_frac as f64);
        for (i, seg) in body.iter().enumerate() {
            if seg.typ == Crashed {
                styles.push(SegmentStyle::Solid(*DEFAULT_CRASHED_COLOR));
            } else {
                let r = (i + body.missing_front) as f64 + frame_frac as f64;
                let start_hue = self.head_hue + (self.tail_hue - self.head_hue) * r / len as f64;
                let end_hue =
                    self.head_hue + (self.tail_hue - self.head_hue) * (r + 1.) / len as f64;
                match seg.typ {
                    Normal | BlackHole => {
                        styles.push(SegmentStyle::HSLGradient {
                            start_hue,
                            end_hue,
                            lightness: self.lightness,
                        });
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
                        styles.push(SegmentStyle::RGBGradient {
                            start_rgb: invert_rgb(start_hsl.to_rgb()),
                            end_rgb: invert_rgb(end_hsl.to_rgb()),
                        });
                    }
                    Crashed => unreachable!(),
                };
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
    fn segment_styles(&mut self, body: &SnakeBody, frame_frac: f32) -> Vec<SegmentStyle> {
        let mut styles = Vec::with_capacity(body.len());

        let frame_frac = frame_frac as f64;
        let len = and_update_max_len(&mut self.max_len, body.len());
        let len = correct_len(len, body, frame_frac);
        for (i, seg) in body.iter().enumerate() {
            let r = (i + body.missing_front) as f64 + frame_frac;
            let hue = self.head_hue + (self.tail_hue - self.head_hue) * r / len;
            let color = match seg.typ {
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
            };
            styles.push(SegmentStyle::Solid(color));
        }

        styles
    }
}

pub struct Alternating {
    color1: Color,
    color2: Color,
    iteration: bool,
    last_head: Option<HexPoint>,
}

impl Palette for Alternating {
    fn segment_styles(&mut self, body: &SnakeBody, _frame_frac: f32) -> Vec<SegmentStyle> {
        let mut styles = Vec::with_capacity(body.len());

        let head = Some(body[0].pos);
        if head != self.last_head {
            self.last_head = head;
            self.iteration = !self.iteration;
        }
        let expected_mod = if self.iteration { 0 } else { 1 };
        for (i, seg) in body.iter().enumerate() {
            let color = match seg.typ {
                Normal | BlackHole => {
                    if i % 2 == expected_mod {
                        self.color1
                    } else {
                        self.color2
                    }
                }
                Eaten { .. } => {
                    if i % 2 == expected_mod {
                        *DEFAULT_EATEN_COLOR
                    } else {
                        self.color2
                    }
                }
                Crashed => *DEFAULT_CRASHED_COLOR,
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
