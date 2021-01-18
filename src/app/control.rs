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
#[derive(Copy, Clone)]
pub enum KeyboardLayout {
    Qwerty,
    Dvorak,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone)]
pub struct ControlSetup {
    pub layout: KeyboardLayout,
    pub keyboard_side: Side,
    pub hand: Side,
}

impl Into<Controls> for ControlSetup {
    fn into(self) -> Controls {
        use ggez::input::keyboard::KeyCode::*;
        use KeyboardLayout::*;
        match (self.layout, self.keyboard_side, self.hand) {
            (Dvorak, Side::Right, Side::Right) => Controls {
                ul: H,
                u: T,
                ur: N,
                dl: M,
                d: W,
                dr: V,
            },
            (Dvorak, Side::Right, Side::Left) => Controls {
                ul: H,
                u: T,
                ur: N,
                dl: B,
                d: M,
                dr: W,
            },
            (Dvorak, Side::Left, Side::Right) => Controls {
                ul: O,
                u: E,
                ur: U,
                dl: Q,
                d: J,
                dr: K,
            },
            (Dvorak, Side::Left, Side::Left) => Controls {
                ul: O,
                u: E,
                ur: U,
                dl: Semicolon,
                d: Q,
                dr: J,
            },
            (Qwerty, Side::Right, Side::Right) => Controls {
                ul: J,
                u: K,
                ur: L,
                dl: M,
                d: Comma,
                dr: Period,
            },
            (Qwerty, Side::Right, Side::Left) => Controls {
                ul: J,
                u: K,
                ur: L,
                dl: N,
                d: M,
                dr: Comma,
            },
            (Qwerty, Side::Left, Side::Right) => Controls {
                ul: S,
                u: D,
                ur: F,
                dl: X,
                d: C,
                dr: V,
            },
            (Qwerty, Side::Left, Side::Left) => Controls {
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
