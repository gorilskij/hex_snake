//! Keys as the player's keyboard layout names them.
//!
//! Key codes are positional on most platforms (macOS, Windows and the web
//! report where a key is, as if the keyboard were qwerty), so a key that
//! types something is identified by the character it types instead: on
//! dvorak, the key in qwerty's J position is `H`. Keys that type nothing
//! (modifiers, arrows, …) keep their key code. Nothing here knows the layout;
//! the character arrives from the OS right after the key press.

use macroquad::input::utils::{register_input_subscriber, repeat_all_miniquad_input};
use macroquad::input::KeyCode;
use macroquad::miniquad::{EventHandler, KeyMods};

use crate::basic::{Dir, Side};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Key {
    /// A key that types something, by what it types (always uppercase, so
    /// caps lock and shift don't make a different key)
    Char(char),
    /// A key that types nothing
    Code(KeyCode),
}

impl Key {
    /// The key that typed `c`, or `None` if `c` is not something a key is
    /// named by (whitespace, control characters, and the private-use
    /// characters macOS reports for arrows and function keys).
    fn from_char(c: char) -> Option<Self> {
        let private_use = ('\u{E000}'..='\u{F8FF}').contains(&c);
        if c.is_control() || c.is_whitespace() || private_use {
            return None;
        }
        let mut upper = c.to_uppercase();
        // a few characters uppercase to several (ß → SS): keep those as typed
        Some(Self::Char(match (upper.next(), upper.next()) {
            (Some(u), None) => u,
            _ => c,
        }))
    }
}

/// Turns miniquad's input events into [`Key`] presses, in the order they
/// happened.
///
/// Every platform sends a key's code and then, if it types something, the
/// character it types; the two are paired here. Repeats (a held key) are
/// dropped.
pub struct KeyInput {
    subscriber: usize,
}

impl KeyInput {
    pub fn new() -> Self {
        Self { subscriber: register_input_subscriber() }
    }

    /// The key presses since the last call. Call once per frame.
    pub fn poll(&mut self) -> Vec<Key> {
        let mut collector = Collector::default();
        repeat_all_miniquad_input(&mut collector, self.subscriber);
        collector.flush();
        collector.presses
    }
}

#[derive(Default)]
struct Collector {
    presses: Vec<Key>,
    /// The last key down and whether it was a repeat, waiting to see whether
    /// a character follows
    pending: Option<(KeyCode, bool)>,
}

impl Collector {
    fn flush(&mut self) {
        if let Some((code, repeat)) = self.pending.take() {
            if !repeat {
                self.presses.push(Key::Code(code));
            }
        }
    }
}

impl EventHandler for Collector {
    fn update(&mut self) {}

    fn draw(&mut self) {}

    fn key_down_event(&mut self, code: KeyCode, _: KeyMods, repeat: bool) {
        self.flush();
        self.pending = Some((code, repeat));
    }

    fn char_event(&mut self, character: char, _: KeyMods, _: bool) {
        // a character with no key down before it (the web sends Space's twice)
        let Some((code, repeat)) = self.pending.take() else {
            return;
        };
        if !repeat {
            self.presses.push(Key::from_char(character).unwrap_or(Key::Code(code)));
        }
    }
}

/// How a key is shown on its button: an optional `L`/`R` for keys that come
/// in pairs, then the key's name, which is smaller when it's a word rather
/// than a symbol.
pub struct KeyLabel {
    pub side: Option<&'static str>,
    pub name: String,
    pub small: bool,
}

impl Key {
    pub fn label(self) -> KeyLabel {
        use KeyCode::*;

        let code = match self {
            Self::Char(c) => return KeyLabel { side: None, name: c.to_string(), small: false },
            Self::Code(code) => code,
        };

        let mac = cfg!(target_os = "macos");
        let side = match code {
            LeftShift | LeftControl | LeftAlt | LeftSuper => Some("L"),
            RightShift | RightControl | RightAlt | RightSuper => Some("R"),
            _ => None,
        };
        let (symbol, word) = match code {
            LeftShift | RightShift => ("⇧", "Shift"),
            LeftControl | RightControl => ("⌃", "Ctrl"),
            LeftAlt | RightAlt => ("⌥", "Alt"),
            LeftSuper | RightSuper => ("⌘", if cfg!(windows) { "Win" } else { "Super" }),
            CapsLock => ("⇪", "Caps"),
            Tab => ("⇥", "Tab"),
            Enter => ("↩", "Enter"),
            Backspace => ("⌫", "Bksp"),
            Delete => ("⌦", "Del"),
            // the same symbols everywhere
            Left => return symbol("←"),
            Up => return symbol("↑"),
            Right => return symbol("→"),
            Down => return symbol("↓"),
            _ => return KeyLabel { side: None, name: code_name(code).to_string(), small: true },
        };

        if mac {
            KeyLabel { side, name: symbol.to_string(), small: false }
        } else {
            KeyLabel { side, name: word.to_string(), small: true }
        }
    }
}

fn symbol(name: &str) -> KeyLabel {
    KeyLabel { side: None, name: name.to_string(), small: false }
}

/// Every key code by name, for the stored preferences (a key code has no
/// stable number to store)
#[rustfmt::skip]
const CODE_NAMES: &[(KeyCode, &str)] = {
    use KeyCode::*;
    &[
        (Space, "Space"), (Apostrophe, "Apostrophe"), (Comma, "Comma"), (Minus, "Minus"),
        (Period, "Period"), (Slash, "Slash"), (Key0, "Key0"), (Key1, "Key1"), (Key2, "Key2"),
        (Key3, "Key3"), (Key4, "Key4"), (Key5, "Key5"), (Key6, "Key6"), (Key7, "Key7"),
        (Key8, "Key8"), (Key9, "Key9"), (Semicolon, "Semicolon"), (Equal, "Equal"), (A, "A"),
        (B, "B"), (C, "C"), (D, "D"), (E, "E"), (F, "F"), (G, "G"), (H, "H"), (I, "I"),
        (J, "J"), (K, "K"), (L, "L"), (M, "M"), (N, "N"), (O, "O"), (P, "P"), (Q, "Q"),
        (R, "R"), (S, "S"), (T, "T"), (U, "U"), (V, "V"), (W, "W"), (X, "X"), (Y, "Y"),
        (Z, "Z"), (LeftBracket, "LeftBracket"), (Backslash, "Backslash"),
        (RightBracket, "RightBracket"), (GraveAccent, "GraveAccent"), (World1, "World1"),
        (World2, "World2"), (Escape, "Escape"), (Enter, "Enter"), (Tab, "Tab"),
        (Backspace, "Backspace"), (Insert, "Insert"), (Delete, "Delete"), (Right, "Right"),
        (Left, "Left"), (Down, "Down"), (Up, "Up"), (PageUp, "PageUp"), (PageDown, "PageDown"),
        (Home, "Home"), (End, "End"), (CapsLock, "CapsLock"), (ScrollLock, "ScrollLock"),
        (NumLock, "NumLock"), (PrintScreen, "PrintScreen"), (Pause, "Pause"), (F1, "F1"),
        (F2, "F2"), (F3, "F3"), (F4, "F4"), (F5, "F5"), (F6, "F6"), (F7, "F7"), (F8, "F8"),
        (F9, "F9"), (F10, "F10"), (F11, "F11"), (F12, "F12"), (F13, "F13"), (F14, "F14"),
        (F15, "F15"), (F16, "F16"), (F17, "F17"), (F18, "F18"), (F19, "F19"), (F20, "F20"),
        (F21, "F21"), (F22, "F22"), (F23, "F23"), (F24, "F24"), (F25, "F25"), (Kp0, "Kp0"),
        (Kp1, "Kp1"), (Kp2, "Kp2"), (Kp3, "Kp3"), (Kp4, "Kp4"), (Kp5, "Kp5"), (Kp6, "Kp6"),
        (Kp7, "Kp7"), (Kp8, "Kp8"), (Kp9, "Kp9"), (KpDecimal, "KpDecimal"),
        (KpDivide, "KpDivide"), (KpMultiply, "KpMultiply"), (KpSubtract, "KpSubtract"),
        (KpAdd, "KpAdd"), (KpEnter, "KpEnter"), (KpEqual, "KpEqual"), (LeftShift, "LeftShift"),
        (LeftControl, "LeftControl"), (LeftAlt, "LeftAlt"), (LeftSuper, "LeftSuper"),
        (RightShift, "RightShift"), (RightControl, "RightControl"), (RightAlt, "RightAlt"),
        (RightSuper, "RightSuper"), (Menu, "Menu"), (Back, "Back"), (Unknown, "Unknown"),
    ]
};

fn code_name(code: KeyCode) -> &'static str {
    CODE_NAMES
        .iter()
        .find(|(c, _)| *c == code)
        .map_or("?", |(_, name)| name)
}

/// The stored form of a key: `char:J` or `code:LeftShift`
impl Key {
    pub fn to_text(self) -> String {
        match self {
            Self::Char(c) => format!("char:{c}"),
            Self::Code(code) => format!("code:{}", code_name(code)),
        }
    }

    pub fn from_text(text: &str) -> Option<Self> {
        if let Some(c) = text.strip_prefix("char:") {
            let mut chars = c.chars();
            return match (chars.next(), chars.next()) {
                (Some(c), None) => Self::from_char(c),
                _ => None,
            };
        }
        let name = text.strip_prefix("code:")?;
        CODE_NAMES
            .iter()
            .find(|(_, n)| *n == name)
            .map(|(code, _)| Self::Code(*code))
    }
}

/// A player's six keys, one per direction; a direction can be left without a
/// key.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Controls {
    /// Indexed by `Dir as usize` (clockwise from `U`)
    keys: [Option<Key>; 6],
}

impl Controls {
    /// The default keys of the player on the `side` of the keyboard, given as
    /// UL, U, UR, DL, D, DR
    pub fn default_for(side: Side) -> Self {
        let chars = match side {
            Side::Left => ['S', 'D', 'F', 'Z', 'X', 'C'],
            Side::Right => ['J', 'K', 'L', 'M', ',', '.'],
        };
        let mut this = Self { keys: [None; 6] };
        for (dir, c) in [Dir::Ul, Dir::U, Dir::Ur, Dir::Dl, Dir::D, Dir::Dr].into_iter().zip(chars) {
            this.set(dir, Some(Key::Char(c)));
        }
        this
    }

    pub fn get(&self, dir: Dir) -> Option<Key> {
        self.keys[dir as usize]
    }

    pub fn set(&mut self, dir: Dir, key: Option<Key>) {
        self.keys[dir as usize] = key;
    }

    /// The direction `key` is bound to
    pub fn dir_of(&self, key: Key) -> Option<Dir> {
        (0..6).find(|&i| self.keys[i] == Some(key)).map(|i| Dir::from(i as u8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_does_not_make_a_different_key() {
        assert_eq!(Key::from_char('j'), Key::from_char('J'));
        assert_eq!(Key::from_char('é'), Some(Key::Char('É')));
    }

    #[test]
    fn keys_that_type_nothing_are_not_chars() {
        for c in [' ', '\r', '\t', '\u{7f}', '\u{F702}'] {
            assert_eq!(Key::from_char(c), None, "{c:?}");
        }
    }

    #[test]
    fn stored_keys_read_back() {
        for key in [Key::Char('J'), Key::Char(','), Key::Char('='), Key::Code(KeyCode::RightShift)] {
            assert_eq!(Key::from_text(&key.to_text()), Some(key));
        }
        assert_eq!(Key::from_text("code:Nonsense"), None);
    }

    #[test]
    fn default_keys_are_in_direction_order() {
        let right = Controls::default_for(Side::Right);
        assert_eq!(right.get(Dir::Ul), Some(Key::Char('J')));
        assert_eq!(right.get(Dir::Dr), Some(Key::Char('.')));
        assert_eq!(right.dir_of(Key::Char('K')), Some(Dir::U));
    }
}
