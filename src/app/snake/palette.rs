use ggez::graphics::Color;
use hsl::HSL;

use SegmentType::*;

use crate::{
    app::snake::{Body, SegmentType},
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

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub enum PaletteTemplate {
    Solid {
        color: Color,
        eaten: Color,
    },
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
    /// Segments are fixed to the board
    AlternatingFixed {
        color1: Color,
        color2: Color,
    },
    /// Segments travel along with the snake
    Alternating {
        color1: Color,
        color2: Color,
    },
}

// TODO: write a builder
#[allow(dead_code)]
impl PaletteTemplate {
    pub fn solid_white_red() -> Self {
        Self::Solid {
            color: Color::WHITE,
            eaten: Color::RED,
        }
    }

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
        Self::AlternatingFixed {
            color1: Color::WHITE,
            color2: Color::new(0., 0., 0., 0.),
        }
    }

    pub fn zebra() -> Self {
        Self::Alternating {
            color1: Color::WHITE,
            color2: Color { r: 0., g: 0., b: 0., a: 0. },
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
    OkLabGradient {
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
            Self::OkLabGradient { start_hue, lightness, .. } => Self::Solid(Color::from(
                OkLab::from_lch(lightness, 0.5, start_hue).to_rgb(),
            )),
        }
    }
}

pub trait Palette {
    fn segment_styles(&mut self, body: &Body, frame_frac: f32) -> Vec<SegmentStyle>;
    // TODO: refactor as
    //  fn color_at(&mut self, body: &SnakeBody, point: f32, frame_frac: f32) -> Color;
    //  this avoids unnecessary work for hex palette and is called exactly as many times as needed
}

impl From<PaletteTemplate> for Box<dyn Palette> {
    fn from(template: PaletteTemplate) -> Self {
        match template {
            PaletteTemplate::Solid { color, eaten } => Box::new(Solid { color, eaten }),
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
            PaletteTemplate::AlternatingFixed { color1, color2 } => Box::new(AlternatingFixed {
                color1,
                color2,
                iteration: true,
                last_head: None,
            }),
            PaletteTemplate::Alternating { color1, color2 } => {
                Box::new(Alternating { color1, color2 })
            }
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
fn correct_len(len: usize, body: &Body, frame_frac: f64) -> f64 {
    let len = len as f64;
    if let SegmentType::Eaten { original_food, food_left } = body[body.len() - 1].typ {
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

// The palettes...

pub struct Solid {
    color: Color,
    eaten: Color,
}

impl Palette for Solid {
    fn segment_styles(&mut self, body: &Body, _frame_frac: f32) -> Vec<SegmentStyle> {
        let mut styles = Vec::with_capacity(body.len());

        for segment in body.iter() {
            let color = match segment.typ {
                Normal | BlackHole => self.color,
                Eaten { .. } => self.eaten,
                Crashed => *DEFAULT_CRASHED_COLOR,
            };
            styles.push(SegmentStyle::Solid(color));
        }

        styles
    }
}

pub struct RGBGradient {
    head: Color,
    tail: Color,
    eaten: EatenColor,
    max_len: Option<usize>,
}

impl Palette for RGBGradient {
    fn segment_styles(&mut self, body: &Body, frame_frac: f32) -> Vec<SegmentStyle> {
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
    fn segment_styles(&mut self, body: &Body, frame_frac: f32) -> Vec<SegmentStyle> {
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
                    Crashed => styles.push(SegmentStyle::Solid(*DEFAULT_CRASHED_COLOR)),
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
    fn segment_styles(&mut self, body: &Body, frame_frac: f32) -> Vec<SegmentStyle> {
        let mut styles = Vec::with_capacity(body.len());

        let frame_frac = frame_frac as f64;
        let len = and_update_max_len(&mut self.max_len, body.len());
        let len = correct_len(len, body, frame_frac);
        for (i, seg) in body.iter().enumerate() {
            let r = (i + body.missing_front) as f64 + frame_frac;
            let start_hue = self.head_hue + (self.tail_hue - self.head_hue) * r / len as f64;
            let end_hue = self.head_hue + (self.tail_hue - self.head_hue) * (r + 1.) / len as f64;
            match seg.typ {
                Normal | BlackHole => {
                    styles.push(SegmentStyle::OkLabGradient {
                        start_hue,
                        end_hue,
                        lightness: self.lightness,
                    });
                }
                Eaten { .. } => {
                    // invert lightness twice
                    let start_okl = OkLab::from_lch(1. - self.eaten_lightness, 0.5, start_hue);
                    let end_okl = OkLab::from_lch(1. - self.eaten_lightness, 0.5, end_hue);
                    styles.push(SegmentStyle::RGBGradient {
                        start_rgb: invert_rgb(start_okl.to_rgb()),
                        end_rgb: invert_rgb(end_okl.to_rgb()),
                    });
                }
                Crashed => styles.push(SegmentStyle::Solid(*DEFAULT_CRASHED_COLOR)),
            };
        }

        styles
    }
}

pub struct AlternatingFixed {
    color1: Color,
    color2: Color,
    iteration: bool,
    last_head: Option<HexPoint>,
}

impl Palette for AlternatingFixed {
    fn segment_styles(&mut self, body: &Body, _frame_frac: f32) -> Vec<SegmentStyle> {
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

pub struct Alternating {
    color1: Color,
    color2: Color,
}

fn mul_color(color: Color, factor: f32) -> Color {
    Color {
        r: color.r * factor,
        g: color.g * factor,
        b: color.b * factor,
        a: color.a,
    }
}

fn add_colors(color1: Color, color2: Color) -> Color {
    Color {
        r: color1.r + color2.r,
        g: color1.g + color2.g,
        b: color1.b + color2.b,
        a: (color1.a + color2.a) / 2.,
    }
}

impl Palette for Alternating {
    fn segment_styles(&mut self, body: &Body, frame_frac: f32) -> Vec<SegmentStyle> {
        let mut styles = Vec::with_capacity(body.len());

        for (i, seg) in body.iter().enumerate() {
            // How far along the snake we currently are (in units of segments)
            let r = (i + body.missing_front) as f32 + frame_frac;

            let _color = match seg.typ {
                Normal | BlackHole => {
                    // Check whether there is a minimum or maximum within this segment
                    use std::f32::consts::PI;
                    if r % PI <= PI && r % PI + 1. >= PI {
                        let diff = PI - r % PI;
                        let _cutoff = r + diff;

                        unimplemented!();
                    }

                    let ratio1_start = (r.cos() + 1.) / 2.;
                    let ratio1_end = ((r + 1.).cos() + 1.) / 2.;

                    let start_color = add_colors(
                        mul_color(self.color1, ratio1_start),
                        mul_color(self.color2, 1. - ratio1_start),
                    );
                    let end_color = add_colors(
                        mul_color(self.color1, ratio1_end),
                        mul_color(self.color2, 1. - ratio1_end),
                    );

                    styles.push(SegmentStyle::RGBGradient {
                        start_rgb: start_color.to_rgb(),
                        end_rgb: end_color.to_rgb(),
                    });
                }
                Eaten { .. } => {
                    styles.push(SegmentStyle::Solid(*DEFAULT_EATEN_COLOR));
                }
                Crashed => styles.push(SegmentStyle::Solid(*DEFAULT_CRASHED_COLOR)),
            };
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
