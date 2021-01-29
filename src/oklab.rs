#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OkLab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

#[test]
fn test() {
    let rgb = (143, 123, 44);
    assert_eq!(rgb, OkLab::from(rgb).to_rgb());

    // let lab = OkLab::from((25, 131, 58));
    // let (l, c, h) = lab.to_lch();
    // assert_eq!(lab, OkLab::from_lch(l, c, h));
}

impl From<(u8, u8, u8)> for OkLab {
    fn from(rgb: (u8, u8, u8)) -> Self {
        // let r = r as f64 / 255.;
        // let g = g as f64 / 255.;
        // let b = b as f64 / 255.;
        //
        // let mut l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        // let mut m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        // let mut s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
        //
        // l = l.cbrt();
        // m = m.cbrt();
        // s = s.cbrt();
        //
        // OkLab {
        //     l: 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        //     a: 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        //     b: 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
        // }

        let r = rgb.0 as f64 / 255.;
        let g = rgb.1 as f64 / 255.;
        let b = rgb.2 as f64 / 255.;

        let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

        // float l_ = cbrtf(l);
        // float m_ = cbrtf(m);
        // float s_ = cbrtf(s);
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

impl OkLab {
    pub fn from_lch(lightness: f64, chroma: f64, hue: f64) -> Self {
        // deg -> rad
        let hue = hue / 360. * 2. * std::f64::consts::PI;
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
            self.b.atan2(self.a) / (2. * std::f64::consts::PI) * 360.,
        )
    }

    pub fn to_rgb(self) -> (u8, u8, u8) {
        // let OkLab { l, a, b } = self;
        //
        // let mut l = l + 0.3963377774 * a + 0.2158037573 * b;
        // let mut m = l - 0.1055613458 * a - 0.0638541728 * b;
        // let mut s = l - 0.0894841775 * a - 1.2914855480 * b;
        //
        // l = l * l * l;
        // m = m * m * m;
        // s = s * s * s;
        //
        // let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
        // let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
        // let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;
        //
        // let r = (r * 255.) as u8;
        // let g = (g * 255.) as u8;
        // let b = (b * 255.) as u8;
        //
        // (r, g, b)

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
