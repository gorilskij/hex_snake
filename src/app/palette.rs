use std::f32::consts::PI;

use ggez::graphics::{Color, BLACK, WHITE};
use hsl::HSL;

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

pub trait SnakePainter {
    fn paint_segment(&mut self, seg: usize, len: usize) -> Color;
}

impl SnakePainter for Color {
    fn paint_segment(&mut self, _: usize, _: usize) -> Color {
        *self
    }
}

impl<T: Fn(usize, usize) -> Color> SnakePainter for T {
    fn paint_segment(&mut self, seg: usize, len: usize) -> Color {
        self(seg, len)
    }
}

pub struct SnakePalette {
    pub segment_color: Box<dyn SnakePainter>,
    pub eaten_color: Box<dyn SnakePainter>,
    pub crashed_color: Color,
    pub portal_color: Color,
}

lazy_static! {
    static ref DEFAULT_EATEN_COLOR: Color = Color::from_rgb(0, 255, 128);
    static ref DEFAULT_CRASHED_COLOR: Color = Color::from_rgb(255, 0, 128);
    static ref DEFAULT_PORTAL_COLOR: Color = Color::from_rgb(245, 192, 64);
}

#[derive(Default)]
pub struct PersistentRainbow {
    max_len: usize,
}

impl SnakePainter for PersistentRainbow {
    fn paint_segment(&mut self, seg: usize, len: usize) -> Color {
        if len > self.max_len {
            self.max_len = len;
        }
        let hue = 273. * seg as f64 / self.max_len as f64;
        let hsl = HSL {
            h: hue,
            s: 1.,
            l: 0.4,
        };
        Color::from(hsl.to_rgb())
    }
}

pub struct InvertRainbow<T: SnakePainter>(T);

impl<T: SnakePainter> SnakePainter for InvertRainbow<T> {
    fn paint_segment(&mut self, seg: usize, len: usize) -> Color {
        let color = self.0.paint_segment(seg, len);
        let (r, g, b) = color.to_rgb();
        let mut hsl = HSL::from_rgb(&[r, g, b]);
        hsl.h = (hsl.h + 180.) % 360.;
        Color::from(hsl.to_rgb())
    }
}

pub struct InvertRGB<T: SnakePainter>(T);

impl<T: SnakePainter> SnakePainter for InvertRGB<T> {
    fn paint_segment(&mut self, seg: usize, len: usize) -> Color {
        let color = self.0.paint_segment(seg, len);
        let (r, g, b) = color.to_rgb();
        Color::from_rgb(255 - r, 255 - g, 255 - b)
    }
}

#[allow(dead_code)]
impl SnakePalette {
    pub fn gradient(head: Color, tail: Color) -> Self {
        Self {
            segment_color: Box::new(move |seg, len| {
                let head_ratio = 1. - seg as f32 / (len - 1) as f32;
                let tail_ratio = 1. - head_ratio;
                Color {
                    r: head_ratio * head.r + tail_ratio * tail.r,
                    g: head_ratio * head.g + tail_ratio * tail.g,
                    b: head_ratio * head.b + tail_ratio * tail.b,
                    a: 1.,
                }
            }),
            eaten_color: Box::new(move |_, _| *DEFAULT_EATEN_COLOR),
            crashed_color: *DEFAULT_CRASHED_COLOR,
            portal_color: *DEFAULT_PORTAL_COLOR,
        }
    }

    pub fn gray_gradient() -> Self {
        Self::gradient(gray!(0.72), gray!(0.25))
    }

    pub fn rainbow() -> Self {
        Self {
            segment_color: Box::new(|seg, len| {
                let hue = 273. * seg as f64 / len as f64;
                let hsl = HSL {
                    h: hue,
                    s: 1.,
                    l: 0.4,
                };
                Color::from(hsl.to_rgb())
            }),
            eaten_color: Box::new(*DEFAULT_EATEN_COLOR),
            crashed_color: *DEFAULT_CRASHED_COLOR,
            portal_color: *DEFAULT_PORTAL_COLOR,
        }
    }

    pub fn persistent_rainbow() -> Self {
        Self {
            segment_color: Box::new(PersistentRainbow::default()),
            eaten_color: Box::new(InvertRGB(PersistentRainbow::default())),
            crashed_color: *DEFAULT_CRASHED_COLOR,
            portal_color: *DEFAULT_PORTAL_COLOR,
        }
    }

    pub fn checker(on_step: usize, off_step: usize) -> Self {
        Self {
            segment_color: Box::new(move |seg, len| {
                if seg % (on_step + off_step) < on_step || seg == len - 1 {
                    WHITE
                } else {
                    BLACK
                }
            }),
            eaten_color: Box::new(*DEFAULT_EATEN_COLOR),
            crashed_color: *DEFAULT_CRASHED_COLOR,
            portal_color: *DEFAULT_PORTAL_COLOR,
        }
    }

    pub fn sin(period: usize) -> Self {
        Self {
            segment_color: Box::new(move |seg, _| {
                let x = seg as f32 * 2. * PI / period as f32;
                let l = (x.sin() + 1.) / 2.;
                Color {
                    r: l,
                    g: l,
                    b: l,
                    a: 1.,
                }
            }),
            eaten_color: Box::new(*DEFAULT_EATEN_COLOR),
            crashed_color: *DEFAULT_CRASHED_COLOR,
            portal_color: *DEFAULT_PORTAL_COLOR,
        }
    }

    pub fn rainbow_sin(period: usize) -> Self {
        Self {
            segment_color: Box::new(move |seg, _| {
                let x = seg as f32 * 2. * PI / period as f32;
                let l = (x.sin() + 1.) / 2.;
                let h = (x / (2. * PI)).floor() * 30.;
                let hsl = HSL {
                    h: h as f64 % 360.,
                    s: 1.,
                    l: l as f64 / 2.,
                };
                Color::from(hsl.to_rgb())
            }),
            eaten_color: Box::new(*DEFAULT_EATEN_COLOR),
            crashed_color: *DEFAULT_CRASHED_COLOR,
            portal_color: *DEFAULT_PORTAL_COLOR,
        }
    }

    pub fn tropical() -> Self {
        Self::gradient(Color::from_rgb(16, 169, 224), Color::from_rgb(227, 173, 11))
    }
}

pub struct GamePalette {
    pub grid_thickness: f32,
    pub border_thickness: f32,

    pub background_color: Color,
    pub grid_color: Color,
    pub border_color: Color,
    pub apple_color: Color,
}

#[allow(dead_code)]
impl GamePalette {
    pub fn light() -> Self {
        todo!()
        // Self {
        //     line_thickness: 2.,
        //     background_color: WHITE,
        //     foreground_color: BLACK,
        //     apple_fill_color: Color::from_rgb(255, 0, 0),
        //     snake_color: (BLACK, gray!(0.5)),
        //     snake1_color: (
        //         Color::from_rgb(0, 192, 0),
        //         Color::from_rgb(0, 64, 0),
        //     ),
        //     snake2_color: (
        //         Color::from_rgb(16, 169, 224),
        //         Color::from_rgb(12, 129, 171),
        //     ),
        //     crash_color: Color::from_rgb(255, 0, 128),
        //     eaten_color: Color::from_rgb(0, 255, 128),
        //     teleported_color: Color::from_rgb(245, 192, 64),
        // }
    }

    pub fn dark() -> Self {
        Self {
            grid_thickness: 0.5,
            border_thickness: 3.,

            background_color: BLACK,
            grid_color: gray!(0.25),
            border_color: WHITE,
            apple_color: gray!(0.45),
        }
    }
}
