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

/// sRGB's transfer function, from an encoded channel in `[0, 1]` to linear light.
fn decode(c: f64) -> f64 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// And back.
fn encode(c: f64) -> f64 {
    if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.powf(1. / 2.4) - 0.055 }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OkLab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

impl From<(u8, u8, u8)> for OkLab {
    #[allow(clippy::many_single_char_names)]
    fn from(rgb: (u8, u8, u8)) -> Self {
        let r = decode(rgb.0 as f64 / 255.);
        let g = decode(rgb.1 as f64 / 255.);
        let b = decode(rgb.2 as f64 / 255.);

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

    /// Linear-light sRGB, each channel in `[0, 1]` when the color is in gamut.
    #[allow(clippy::many_single_char_names)]
    fn to_linear_rgb(self) -> (f64, f64, f64) {
        let OkLab { l, a, b } = self;

        let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
        let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
        let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        (
            4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
            -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
            -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
        )
    }

    /// Whether the color can be shown in sRGB as it is.
    pub fn in_gamut(self) -> bool {
        const EPSILON: f64 = 1e-9;
        let (r, g, b) = self.to_linear_rgb();
        [r, g, b].iter().all(|c| (-EPSILON..=1. + EPSILON).contains(c))
    }

    /// The most chroma a color of this lightness and hue can have in sRGB.
    pub fn max_chroma(lightness: f64, hue: f64) -> f64 {
        let (mut lo, mut hi) = (0., 0.5);
        for _ in 0..32 {
            let mid = (lo + hi) / 2.;
            if Self::from_lch(lightness, mid, hue).in_gamut() {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lo
    }

    /// sRGB bytes; channels outside the gamut are clipped.
    pub fn to_rgb(self) -> (u8, u8, u8) {
        let (r, g, b) = self.to_linear_rgb();
        let byte = |c: f64| (encode(c.clamp(0., 1.)) * 255.).round() as u8;
        (byte(r), byte(g), byte(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srgb_colors_come_back_as_they_went_in() {
        for rgb in [(0, 0, 0), (255, 255, 255), (255, 0, 0), (12, 200, 99), (128, 128, 128)] {
            assert_eq!(OkLab::from(rgb).to_rgb(), rgb);
        }
    }

    #[test]
    fn white_is_lightness_one_and_grey_has_no_chroma() {
        let white = OkLab::from((255, 255, 255));
        assert!((white.l - 1.).abs() < 1e-3 && white.a.abs() < 1e-3 && white.b.abs() < 1e-3);
        let (_, chroma, _) = OkLab::from((128, 128, 128)).to_lch();
        assert!(chroma < 1e-3);
    }

    #[test]
    fn the_most_chroma_there_is_is_just_in_gamut() {
        for hue in [0., 60., 120., 200., 264., 320.] {
            let max = OkLab::max_chroma(0.75, hue);
            assert!(OkLab::from_lch(0.75, max, hue).in_gamut());
            assert!(!OkLab::from_lch(0.75, max + 0.01, hue).in_gamut());
        }
    }
}
