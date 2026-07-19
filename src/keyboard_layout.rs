use macroquad::input::KeyCode::{self, *};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Layout {
    Qwerty,
    Dvorak,
}

/// Constants are top-to-bottom, left-to-right, Mac-centric
type LayoutSpec = [KeyCode; 63];
const QWERTY_LAYOUT: LayoutSpec = [
    Escape,
    GraveAccent,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,
    Minus,
    Equal,
    Back,
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    LeftBracket,
    RightBracket,
    Backslash,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Semicolon,
    Apostrophe,
    Enter,
    LeftShift,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Period,
    Slash,
    RightShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    Space,
    RightSuper,
    RightAlt,
    Left,
    Up,
    Down,
    Right,
];
const DVORAK_LAYOUT: LayoutSpec = [
    Escape,
    GraveAccent,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,
    LeftBracket,
    RightBracket,
    Back,
    Tab,
    Apostrophe,
    Comma,
    Period,
    P,
    Y,
    F,
    G,
    C,
    R,
    L,
    Slash,
    Equal,
    Backslash,
    A,
    O,
    E,
    U,
    I,
    D,
    H,
    T,
    N,
    S,
    Minus,
    Enter,
    LeftShift,
    Semicolon,
    Q,
    J,
    K,
    X,
    B,
    M,
    W,
    V,
    Z,
    RightShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    Space,
    RightSuper,
    RightAlt,
    Left,
    Up,
    Down,
    Right,
];

const fn get_layout(layout: Layout) -> &'static LayoutSpec {
    match layout {
        Layout::Qwerty => &QWERTY_LAYOUT,
        Layout::Dvorak => &DVORAK_LAYOUT,
    }
}

pub struct LayoutConverter {
    from: &'static LayoutSpec,
    to: &'static LayoutSpec,
}

impl LayoutConverter {
    pub fn new(from: Layout, to: Layout) -> Self {
        Self {
            from: get_layout(from),
            to: get_layout(to),
        }
    }

    pub fn cvt(&self, keycode: KeyCode) -> KeyCode {
        if self.from == self.to {
            return keycode;
        }
        let idx = self
            .from
            .iter()
            .position(|k| *k == keycode)
            .unwrap_or_else(|| panic!("Unknown keycode: {keycode:?}"));
        self.to[idx]
    }
}
