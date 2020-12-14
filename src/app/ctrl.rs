use ggez::input::keyboard::KeyCode;

#[derive(Copy, Clone)]
pub struct Controls {
    pub u: KeyCode,
    pub d: KeyCode,
    pub ul: KeyCode,
    pub ur: KeyCode,
    pub dl: KeyCode,
    pub dr: KeyCode,
}

pub enum CtrlLayout { Qwerty, Dvorak }
pub enum CtrlSide { LeftSide, RightSide }
pub struct Ctrl {
    pub layout: CtrlLayout,
    pub keyboard_side: CtrlSide,
    pub hand: CtrlSide,
}

impl Into<Controls> for Ctrl {
    fn into(self) -> Controls {
        use CtrlLayout::*;
        use CtrlSide::*;
        use ggez::input::keyboard::KeyCode::*;
        match (self. layout, self.keyboard_side, self.hand) {
            (Dvorak, RightSide, RightSide) => Controls { ul: H, u: T, ur: N, dl: M, d: W, dr: V },
            (Dvorak, RightSide, LeftSide) => Controls { ul: H, u: T, ur: N, dl: B, d: M, dr: W },
            (Dvorak, LeftSide, RightSide) => Controls { ul: O, u: E, ur: U, dl: Q, d: J, dr: K },
            (Dvorak, LeftSide, LeftSide) => Controls { ul: O, u: E, ur: U, dl: Semicolon, d: Q, dr: J },
            (Qwerty, RightSide, RightSide) => Controls { ul: J, u: K, ur: L, dl: M, d: Comma, dr: Period },
            (Qwerty, RightSide, LeftSide) => Controls { ul: J, u: K, ur: L, dl: N, d: M, dr: Comma },
            (Qwerty, LeftSide, RightSide) => Controls { ul: S, u: D, ur: F, dl: X, d: C, dr: V },
            (Qwerty, LeftSide, LeftSide) => Controls { ul: S, u: D, ur: F, dl: Z, d: X, dr: C },
        }
    }
}
