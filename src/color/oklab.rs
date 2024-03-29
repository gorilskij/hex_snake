// Copyright (c) 2020 Björn Ottosson
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is furnished to do
// so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::f64::consts::TAU;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OkLab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

impl From<(u8, u8, u8)> for OkLab {
    #[allow(clippy::many_single_char_names)]
    fn from(rgb: (u8, u8, u8)) -> Self {
        let r = rgb.0 as f64 / 255.;
        let g = rgb.1 as f64 / 255.;
        let b = rgb.2 as f64 / 255.;

        let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        Self {
            l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
            a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
            b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
        }
    }
}

#[allow(dead_code)]
impl OkLab {
    pub fn from_lch(lightness: f64, chroma: f64, hue: f64) -> Self {
        // deg -> rad
        let hue = hue / 360. * TAU;
        Self {
            l: lightness,
            a: chroma * hue.cos(),
            b: chroma * hue.sin(),
        }
    }

    pub fn to_lch(self) -> (f64, f64, f64) {
        (
            self.l,
            (self.a * self.a + self.b * self.b).sqrt(),
            self.b.atan2(self.a) / TAU * 360.,
        )
    }

    #[allow(clippy::many_single_char_names)]
    pub fn to_rgb(self) -> (u8, u8, u8) {
        let OkLab { l, a, b } = self;

        let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
        let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
        let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
        let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
        let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

        ((r * 255.) as u8, (g * 255.) as u8, (b * 255.) as u8)
    }
}
