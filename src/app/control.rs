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

#[allow(dead_code)]
pub enum KeyboardLayout {
    Qwerty,
    Dvorak,
}

#[allow(dead_code)]
pub enum Side {
    LeftSide,
    RightSide,
}

pub struct ControlSetup {
    pub layout: KeyboardLayout,
    pub keyboard_side: Side,
    pub hand: Side,
}

impl Into<Controls> for ControlSetup {
    fn into(self) -> Controls {
        use ggez::input::keyboard::KeyCode::*;
        use KeyboardLayout::*;
        use Side::*;
        match (self.layout, self.keyboard_side, self.hand) {
            (Dvorak, RightSide, RightSide) => Controls {
                ul: H,
                u: T,
                ur: N,
                dl: M,
                d: W,
                dr: V,
            },
            (Dvorak, RightSide, LeftSide) => Controls {
                ul: H,
                u: T,
                ur: N,
                dl: B,
                d: M,
                dr: W,
            },
            (Dvorak, LeftSide, RightSide) => Controls {
                ul: O,
                u: E,
                ur: U,
                dl: Q,
                d: J,
                dr: K,
            },
            (Dvorak, LeftSide, LeftSide) => Controls {
                ul: O,
                u: E,
                ur: U,
                dl: Semicolon,
                d: Q,
                dr: J,
            },
            (Qwerty, RightSide, RightSide) => Controls {
                ul: J,
                u: K,
                ur: L,
                dl: M,
                d: Comma,
                dr: Period,
            },
            (Qwerty, RightSide, LeftSide) => Controls {
                ul: J,
                u: K,
                ur: L,
                dl: N,
                d: M,
                dr: Comma,
            },
            (Qwerty, LeftSide, RightSide) => Controls {
                ul: S,
                u: D,
                ur: F,
                dl: X,
                d: C,
                dr: V,
            },
            (Qwerty, LeftSide, LeftSide) => Controls {
                ul: S,
                u: D,
                ur: F,
                dl: Z,
                d: X,
                dr: C,
            },
        }
    }
}
