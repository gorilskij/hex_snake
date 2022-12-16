use ggez::graphics;
use hsl::HSL;

use crate::basic::HexPoint;
use crate::color::oklab::OkLab;
use crate::color::to_color::ToColor;
use crate::color::Color;
use crate::snake::{Body, SegmentType};
use crate::support::limits::Limits;

macro_rules! gray {
    ($lightness:expr) => {
        gray!($lightness, 1.)
    };
    ($lightness:expr, $opacity:expr) => {
        crate::color::Color(ggez::graphics::Color {
            r: $lightness,
            g: $lightness,
            b: $lightness,
            a: $opacity,
        })
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

// fn invert_rgb(rgb: (u8, u8, u8)) -> (u8, u8, u8) {
//     (255 - rgb.0, 255 - rgb.1, 255 - rgb.2)
// }

fn invert_rgb(color: Color) -> Color {
    Color(graphics::Color {
        r: 1. - color.r,
        g: 1. - color.g,
        b: 1. - color.b,
        a: color.a,
    })
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

    pub fn gray_gradient(opacity: f32, persistent: bool) -> Self {
        Self::rgb_gradient(gray!(0.72, opacity), gray!(0.25, opacity), None, persistent)
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
            color2: Color::TRANSPARENT,
        }
    }

    pub fn zebra() -> Self {
        Self::Alternating {
            color1: Color::WHITE,
            color2: Color::TRANSPARENT,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SegmentStyle {
    Solid(Color),
    RGBGradient {
        start_color: Color,
        end_color: Color,
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
    pub fn first_color(&self) -> Color {
        match self {
            &Self::Solid(color) => color,
            &Self::RGBGradient { start_color: start_rgb, .. } => start_rgb,
            &Self::HSLGradient { start_hue, lightness, .. } => {
                HSL { h: start_hue, s: 1., l: lightness }.to_color()
            }
            &Self::OkLabGradient { start_hue, lightness, .. } => {
                OkLab::from_lch(lightness, 0.5, start_hue).to_color()
            }
        }
    }

    pub fn color_at_fraction(&self) -> Box<dyn Fn(f64) -> Color> {
        match self {
            &SegmentStyle::Solid(color) => Box::new(move |_| color),
            &SegmentStyle::RGBGradient { start_color, end_color } => {
                Box::new(move |f| f * start_color + (1. - f) * end_color)
            }
            &SegmentStyle::HSLGradient { start_hue, end_hue, lightness } => Box::new(move |f| {
                HSL {
                    h: f * start_hue + (1. - f) * end_hue,
                    s: 1.,
                    l: lightness,
                }
                .to_color()
            }),
            &SegmentStyle::OkLabGradient { start_hue, end_hue, lightness } => Box::new(move |f| {
                OkLab::from_lch(lightness, 0.5, f * start_hue + (1. - f) * end_hue).to_color()
            }),
        }
    }
}

pub trait Palette: Send + Sync {
    fn segment_styles(&mut self, body: &Body, frame_fraction: f32) -> Vec<SegmentStyle>;
    // TODO: refactor as
    //  fn color_at(&mut self, body: &SnakeBody, point: f32, frame_fraction: f32) -> Color;
    //  this avoids unnecessary work for hex palette and is called exactly as many times as needed
}

impl From<PaletteTemplate> for Box<dyn Palette + Send + Sync> {
    fn from(template: PaletteTemplate) -> Self {
        match template {
            PaletteTemplate::Solid { color, eaten } => Box::new(Solid { color, eaten }),
            PaletteTemplate::RGBGradient { head, tail, eaten, persistent } => {
                Box::new(RGBGradient {
                    head_color: head,
                    tail_color: tail,
                    eaten,
                    max_len: persistent.then_some(0),
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
                max_len: persistent.then_some(0),
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
                max_len: persistent.then_some(0),
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
//  this is used to implement persistency
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
fn correct_len(len: usize, body: &Body, frame_fraction: f64) -> f64 {
    let len = len as f64;
    if let SegmentType::Eaten { original_food, food_left } =
        body.segments.last().unwrap().segment_type
    {
        // Correct for eaten segment at the tail and
        //  fractional segment at the head (the eaten
        //  segment reduces in size more slowly than
        //  the head segment grows)

        // The actual visual length of the eaten segment
        //  at the tail of the snake
        let eaten_segment_frac =
            (food_left as f64 + 1. - frame_fraction) / (original_food + 1) as f64;

        len - 1. + eaten_segment_frac + frame_fraction
    } else if body.grow > 0 {
        // If growth is happening for a reason other
        //  than eating (such as at the beginning of
        //  the game), correct only for the head
        len + frame_fraction
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
    fn segment_styles(&mut self, body: &Body, _frame_fraction: f32) -> Vec<SegmentStyle> {
        use SegmentType::*;

        let mut styles = Vec::with_capacity(body.visible_len());

        for segment in body.segments.iter() {
            let color = match segment.segment_type {
                Normal | BlackHole { .. } => self.color,
                Eaten { .. } => self.eaten,
                Crashed => *DEFAULT_CRASHED_COLOR,
            };
            styles.push(SegmentStyle::Solid(color));
        }

        styles
    }
}

pub struct RGBGradient {
    head_color: Color,
    tail_color: Color,
    eaten: EatenColor,
    max_len: Option<usize>,
}

impl Palette for RGBGradient {
    fn segment_styles(&mut self, body: &Body, frame_fraction: f32) -> Vec<SegmentStyle> {
        use SegmentType::*;

        let mut styles = Vec::with_capacity(body.visible_len());

        let logical_len = and_update_max_len(&mut self.max_len, body.logical_len());
        let logical_len = correct_len(logical_len, body, frame_fraction as f64);
        for (i, seg) in body.segments.iter().enumerate() {
            if seg.segment_type == Crashed {
                styles.push(SegmentStyle::Solid(*DEFAULT_CRASHED_COLOR));
            } else {
                let r = (i + body.missing_front) as f64 + frame_fraction as f64;
                let start_color =
                    self.head_color + (self.tail_color - self.head_color) * r / logical_len;
                let end_color =
                    self.head_color + (self.tail_color - self.head_color) * (r + 1.) / logical_len;

                match seg.segment_type {
                    Normal | BlackHole { .. } => {
                        styles.push(SegmentStyle::RGBGradient { start_color, end_color });
                    }
                    Eaten { .. } => {
                        styles.push(SegmentStyle::RGBGradient {
                            start_color: invert_rgb(start_color),
                            end_color: invert_rgb(end_color),
                        });
                    }
                    Crashed => styles.push(SegmentStyle::Solid(*DEFAULT_CRASHED_COLOR)),
                };
            }
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
    fn segment_styles(&mut self, body: &Body, frame_fraction: f32) -> Vec<SegmentStyle> {
        use SegmentType::*;

        let mut styles = Vec::with_capacity(body.visible_len());

        let logical_len = and_update_max_len(&mut self.max_len, body.logical_len());
        let logical_len = correct_len(logical_len, body, frame_fraction as f64);
        for (i, seg) in body.segments.iter().enumerate() {
            if seg.segment_type == Crashed {
                styles.push(SegmentStyle::Solid(*DEFAULT_CRASHED_COLOR));
            } else {
                let r = (i + body.missing_front) as f64 + frame_fraction as f64;
                let start_hue = self.head_hue + (self.tail_hue - self.head_hue) * r / logical_len;
                let end_hue =
                    self.head_hue + (self.tail_hue - self.head_hue) * (r + 1.) / logical_len;

                match seg.segment_type {
                    Normal | BlackHole { .. } => {
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
                            start_color: invert_rgb(start_hsl.to_color()),
                            end_color: invert_rgb(end_hsl.to_color()),
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
    fn segment_styles(&mut self, body: &Body, frame_fraction: f32) -> Vec<SegmentStyle> {
        use SegmentType::*;

        let mut styles = Vec::with_capacity(body.visible_len());

        let frame_fraction = frame_fraction as f64;
        let logical_len = and_update_max_len(&mut self.max_len, body.logical_len());
        let logical_len = correct_len(logical_len, body, frame_fraction);
        for (i, seg) in body.segments.iter().enumerate() {
            let r = (i + body.missing_front) as f64 + frame_fraction;
            let start_hue = self.head_hue + (self.tail_hue - self.head_hue) * r / logical_len;
            let end_hue = self.head_hue + (self.tail_hue - self.head_hue) * (r + 1.) / logical_len;
            match seg.segment_type {
                Normal | BlackHole { .. } => {
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
                        start_color: invert_rgb(start_okl.to_color()),
                        end_color: invert_rgb(end_okl.to_color()),
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
    fn segment_styles(&mut self, body: &Body, _frame_fraction: f32) -> Vec<SegmentStyle> {
        use SegmentType::*;

        let mut styles = Vec::with_capacity(body.visible_len());

        let head = Some(body.segments[0].pos);
        if head != self.last_head {
            self.last_head = head;
            self.iteration = !self.iteration;
        }
        let expected_mod = !self.iteration as usize;
        for (i, seg) in body.segments.iter().enumerate() {
            let color = match seg.segment_type {
                Normal | BlackHole { .. } => {
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

impl Palette for Alternating {
    fn segment_styles(&mut self, body: &Body, frame_fraction: f32) -> Vec<SegmentStyle> {
        use SegmentType::*;

        let mut styles = Vec::with_capacity(body.visible_len());

        for (i, seg) in body.segments.iter().enumerate() {
            // How far along the snake we currently are (in units of segments)
            let r = (i + body.missing_front) as f64 + frame_fraction as f64;

            match seg.segment_type {
                Normal | BlackHole { .. } => {
                    // Check whether there is a minimum or maximum within this segment
                    use std::f64::consts::PI;
                    if r % PI <= PI && r % PI + 1. >= PI {
                        let diff = PI - r % PI;
                        let _cutoff = r + diff;

                        unimplemented!();
                    }

                    let ratio1_start = (r.cos() + 1.) / 2.;
                    let ratio1_end = ((r + 1.).cos() + 1.) / 2.;

                    let start_color =
                        ratio1_start * self.color1 + (1. - ratio1_start) * self.color2;
                    let end_color = ratio1_end * self.color1 + (1. - ratio1_end) * self.color2;

                    styles.push(SegmentStyle::RGBGradient { start_color, end_color });
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
