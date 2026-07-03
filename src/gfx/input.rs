//! Keyboard/mouse compat. `KeyCode` mirrors the ggez/winit variant names the
//! game already pattern-matches on, so `keyboard_layout.rs`, `keyboard_control.rs`
//! and `game.rs` compile unchanged. A `From<macroquad::KeyCode>` mapping
//! isolates the naming differences to this one file.

pub mod keyboard {
    use macroquad::input::KeyCode as MqKeyCode;

    /// Subset of physical keys the game references, named to match ggez.
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    #[allow(dead_code)]
    pub enum KeyCode {
        Escape, Grave,
        Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0,
        Minus, Equals, Back, Tab,
        Q, W, E, R, T, Y, U, I, O, P, LBracket, RBracket, Backslash,
        A, S, D, F, G, H, J, K, L, Semicolon, Apostrophe, Return,
        LShift, Z, X, C, V, B, N, M, Comma, Period, Slash, RShift,
        LControl, LAlt, LWin, Space, RWin, RAlt,
        Left, Up, Down, Right,
    }

    impl KeyCode {
        /// Translate a macroquad key into our key set, or `None` for keys the
        /// game doesn't care about.
        pub fn from_macroquad(k: MqKeyCode) -> Option<Self> {
            use KeyCode::*;
            Some(match k {
                MqKeyCode::Escape => Escape,
                MqKeyCode::GraveAccent => Grave,
                MqKeyCode::Key1 => Key1,
                MqKeyCode::Key2 => Key2,
                MqKeyCode::Key3 => Key3,
                MqKeyCode::Key4 => Key4,
                MqKeyCode::Key5 => Key5,
                MqKeyCode::Key6 => Key6,
                MqKeyCode::Key7 => Key7,
                MqKeyCode::Key8 => Key8,
                MqKeyCode::Key9 => Key9,
                MqKeyCode::Key0 => Key0,
                MqKeyCode::Minus => Minus,
                MqKeyCode::Equal => Equals,
                MqKeyCode::Backspace => Back,
                MqKeyCode::Tab => Tab,
                MqKeyCode::Q => Q,
                MqKeyCode::W => W,
                MqKeyCode::E => E,
                MqKeyCode::R => R,
                MqKeyCode::T => T,
                MqKeyCode::Y => Y,
                MqKeyCode::U => U,
                MqKeyCode::I => I,
                MqKeyCode::O => O,
                MqKeyCode::P => P,
                MqKeyCode::LeftBracket => LBracket,
                MqKeyCode::RightBracket => RBracket,
                MqKeyCode::Backslash => Backslash,
                MqKeyCode::A => A,
                MqKeyCode::S => S,
                MqKeyCode::D => D,
                MqKeyCode::F => F,
                MqKeyCode::G => G,
                MqKeyCode::H => H,
                MqKeyCode::J => J,
                MqKeyCode::K => K,
                MqKeyCode::L => L,
                MqKeyCode::Semicolon => Semicolon,
                MqKeyCode::Apostrophe => Apostrophe,
                MqKeyCode::Enter => Return,
                MqKeyCode::LeftShift => LShift,
                MqKeyCode::Z => Z,
                MqKeyCode::X => X,
                MqKeyCode::C => C,
                MqKeyCode::V => V,
                MqKeyCode::B => B,
                MqKeyCode::N => N,
                MqKeyCode::M => M,
                MqKeyCode::Comma => Comma,
                MqKeyCode::Period => Period,
                MqKeyCode::Slash => Slash,
                MqKeyCode::RightShift => RShift,
                MqKeyCode::LeftControl => LControl,
                MqKeyCode::LeftAlt => LAlt,
                MqKeyCode::LeftSuper => LWin,
                MqKeyCode::Space => Space,
                MqKeyCode::RightSuper => RWin,
                MqKeyCode::RightAlt => RAlt,
                MqKeyCode::Left => Left,
                MqKeyCode::Up => Up,
                MqKeyCode::Down => Down,
                MqKeyCode::Right => Right,
                _ => return None,
            })
        }
    }

    /// Mirrors `ggez::input::keyboard::KeyInput` (the part the game reads).
    #[derive(Copy, Clone, Debug)]
    pub struct KeyInput {
        pub keycode: Option<KeyCode>,
    }
}

pub mod mouse {
    use macroquad::input::show_mouse;

    use crate::gfx::Context;

    pub fn set_cursor_hidden(_ctx: &mut Context, hidden: bool) {
        show_mouse(!hidden);
    }
}
