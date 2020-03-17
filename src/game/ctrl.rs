use ggez::input::keyboard::KeyCode::*;
use ggez::input::keyboard::KeyCode;


pub struct Ctrl {
    pub u: KeyCode,
    pub d: KeyCode,
    pub ul: KeyCode,
    pub ur: KeyCode,
    pub dl: KeyCode,
    pub dr: KeyCode,
}


// CTRL_{layout}_{location on keyboard}_{hand used to play}
// DVORAK
pub const CTRL_DVORAK_RIGHT_RIGHT: Ctrl = Ctrl {
    ul: H,
    u: T,
    ur: N,
    dl: M,
    d: W,
    dr: V,
};

pub const CTRL_DVORAK_RIGHT_LEFT: Ctrl = Ctrl {
    ul: H,
    u: T,
    ur: N,
    dl: B,
    d: M,
    dr: W,
};

pub const CTRL_DVORAK_LEFT_RIGHT: Ctrl = Ctrl {
    ul: O,
    u: E,
    ur: U,
    dl: Q,
    d: J,
    dr: K,
};

pub const CTRL_DVORAK_LEFT_LEFT: Ctrl = Ctrl {
    ul: O,
    u: E,
    ur: U,
    dl: Semicolon,
    d: Q,
    dr: J,
};

// QWERTY
pub const CTRL_QUERTY_RIGHT_RIGHT: Ctrl = Ctrl {
    ul: J,
    u: K,
    ur: L,
    dl: M,
    d: Comma,
    dr: Period,
};

pub const CTRL_QUERTY_RIGHT_LEFT: Ctrl = Ctrl {
    ul: J,
    u: K,
    ur: L,
    dl: N,
    d: M,
    dr: Comma,
};

pub const CTRL_QUERTY_LEFT_RIGHT: Ctrl = Ctrl {
    ul: S,
    u: D,
    ur: F,
    dl: X,
    d: C,
    dr: V,
};

pub const CTRL_QUERTY_LEFT_LEFT: Ctrl = Ctrl {
    ul: S,
    u: D,
    ur: F,
    dl: Z,
    d: X,
    dr: C,
};

#[macro_export]
// pseudo-constructor
macro_rules! ctrl {
    // permutations
    (layout: $l:tt, side: $s:tt, hand: $h:tt $(,)?) => { ctrl!(@ $l, $s, $h) };
    (layout: $l:tt, hand: $h:tt, side: $s:tt $(,)?) => { ctrl!(@ $l, $s, $h) };
    (side: $s:tt, layout: $l:tt, hand: $h:tt $(,)?) => { ctrl!(@ $l, $s, $h) };
    (side: $s:tt, hand: $h:tt, layout: $l:tt $(,)?) => { ctrl!(@ $l, $s, $h) };
    (hand: $h:tt, layout: $l:tt, side: $s:tt $(,)?) => { ctrl!(@ $l, $s, $h) };
    (hand: $h:tt, side: $s:tt, layout: $l:tt $(,)?) => { ctrl!(@ $l, $s, $h) };

    (@ dvorak, right, right) => { crate::game::ctrl::CTRL_DVORAK_RIGHT_RIGHT };
    (@ dvorak, right, left) => { crate::game::ctrl::CTRL_DVORAK_RIGHT_LEFT };
    (@ dvorak, left, right) => { crate::game::ctrl::CTRL_DVORAK_LEFT_RIGHT };
    (@ dvorak, left, left) => { crate::game::ctrl::CTRL_DVORAK_LEFT_LEFT };
}
