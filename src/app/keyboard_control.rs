use crate::basic::Side;
use crate::keyboard::{Layout, LayoutConverter};
use ggez::input::keyboard::KeyCode::{self, *};

#[derive(Copy, Clone)]
pub struct Controls {
    pub u: KeyCode,
    pub d: KeyCode,
    pub ul: KeyCode,
    pub ur: KeyCode,
    pub dl: KeyCode,
    pub dr: KeyCode,
}

impl Controls {
    /// All keybinding specifications are in Qwerty, this
    /// function is used to translate them to other keyboard
    /// layouts (find the key in the new layout which
    /// corresponds to the given key in qwerty)
    fn to_layout(self, layout: Layout) -> Self {
        let c = LayoutConverter::new(Layout::Qwerty, layout);
        Self {
            u: c.cvt(self.u),
            d: c.cvt(self.d),
            ul: c.cvt(self.ul),
            ur: c.cvt(self.ur),
            dl: c.cvt(self.dl),
            dr: c.cvt(self.dr),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ControlSetup {
    pub layout: Layout,
    pub keyboard_side: Side,
    pub hand: Side,
}

impl From<ControlSetup> for Controls {
    fn from(setup: ControlSetup) -> Self {
        #[rustfmt::skip]
        let qwerty_controls = match (setup.keyboard_side, setup.hand) {
            (Side::Right, Side::Right) =>
                Self { ul: J, u: K, ur: L, dl: M, d: Comma, dr: Period },
            (Side::Right, Side::Left) =>
                Self { ul: J, u: K, ur: L, dl: N, d: M, dr: Comma },
            (Side::Left, Side::Right) =>
                Self { ul: S, u: D, ur: F, dl: X, d: C, dr: V },
            (Side::Left, Side::Left) =>
                Self { ul: S, u: D, ur: F, dl: Z, d: X, dr: C },
        };

        qwerty_controls.to_layout(setup.layout)
    }
}
