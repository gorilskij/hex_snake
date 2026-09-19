//! The menus drawn over a screen (the game, or the start screen): the
//! options menu and, from it, the controls menus.
//!
//! The menus change preferences themselves (and save them). What a toggle
//! means beyond its preference, e.g. rebuilding the game's meshes, is left to
//! the screen hosting the menu, which gets a [`MenuEvent`] for it.

use macroquad::input::KeyCode;

use super::controls_menu::{self, KeysAction, KeysScreen, PlayersAction};
use super::options_menu::{self, Line};
use crate::app::key::{Key, KeyPress};
use crate::app::prefs::{DrawGrid, HintStyle, Prefs};
use crate::basic::Side;
use crate::rendering;
use crate::support::flip::Flip;

/// An option in the options menu
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Toggle {
    Grid,
    Border,
    Hints,
    DrawStyle,
    Stats,
    Fps,
    PlayerPath,
    DistanceGrid,
    SpecialApples,
    Autopilot,
}

impl Toggle {
    /// The options that are stored preferences, so they can be set before a
    /// game exists (the distance grid is one too, but it's a debug key)
    pub const PREFS: [Toggle; 7] = [
        Toggle::DrawStyle,
        Toggle::Grid,
        Toggle::Border,
        Toggle::Hints,
        Toggle::PlayerPath,
        Toggle::Stats,
        Toggle::Fps,
    ];

    /// What the option's button says: the setting and its current value. The
    /// autopilot's state is the game's, not a preference, so it's the
    /// caller's to describe.
    pub fn label(self, prefs: &Prefs) -> String {
        let on_off = |on: bool| if on { "on" } else { "off" };
        match self {
            Toggle::Grid => format!(
                "Grid: {}",
                match prefs.draw_grid {
                    DrawGrid::Grid => "lines",
                    DrawGrid::Dots => "dots",
                    DrawGrid::None => "off",
                }
            ),
            Toggle::Border => format!("Border: {}", on_off(prefs.draw_border)),
            Toggle::Hints => format!(
                "Edge hints: {}",
                match prefs.hint_style {
                    HintStyle::Border => "border",
                    HintStyle::Gradient => "gradient",
                    HintStyle::Teleport => "teleport",
                    HintStyle::None => "off",
                }
            ),
            Toggle::DrawStyle => format!(
                "Snake style: {}",
                match prefs.draw_style {
                    rendering::Style::Smooth => "smooth",
                    rendering::Style::Hexagon => "hexagon",
                }
            ),
            Toggle::Stats => format!("Stats: {}", on_off(prefs.display_stats)),
            Toggle::Fps => format!("FPS counter: {}", on_off(prefs.display_fps)),
            Toggle::PlayerPath => format!("Autopilot path: {}", on_off(prefs.draw_player_path)),
            Toggle::DistanceGrid => format!("Distance grid: {}", on_off(prefs.draw_distance_grid)),
            Toggle::SpecialApples => format!("Special apples: {}", on_off(prefs.special_apples)),
            Toggle::Autopilot => "Autopilot".to_string(),
        }
    }

    /// Change the option's preference to its next value, and save it if it's
    /// a stored one. The autopilot has no preference: nothing happens.
    pub fn apply(self, prefs: &mut Prefs) {
        use enum_rotate::EnumRotate;

        match self {
            Toggle::Grid => {
                prefs.draw_grid.rotate_next();
            }
            Toggle::Border => {
                prefs.draw_border.flip();
            }
            Toggle::Hints => {
                prefs.hint_style.rotate_next();
            }
            Toggle::DrawStyle => {
                prefs.draw_style = match prefs.draw_style {
                    rendering::Style::Hexagon => rendering::Style::Smooth,
                    rendering::Style::Smooth => rendering::Style::Hexagon,
                }
            }
            Toggle::Stats => {
                prefs.display_stats.flip();
            }
            Toggle::Fps => {
                prefs.display_fps.flip();
            }
            Toggle::PlayerPath => {
                prefs.draw_player_path.flip();
            }
            Toggle::DistanceGrid => {
                prefs.draw_distance_grid.flip();
            }
            // a game rule rather than a preference: not stored
            Toggle::SpecialApples => {
                prefs.special_apples.flip();
                return;
            }
            Toggle::Autopilot => return,
        }

        // Preferences go to storage the moment one changes: the web build has
        // no reliable moment to flush them later.
        prefs.save();
    }
}

/// Something the host screen may need to act on
pub enum MenuEvent {
    /// The menus were closed
    Closed,
    /// An option was clicked; its preference has already changed
    Toggled(Toggle),
    /// Restart the game (confirmed); the menus are closed
    Restart,
    /// Leave the game for the main menu (confirmed); the menus are closed
    MainMenu,
}

/// An action that asks "are you sure?" first
#[derive(Copy, Clone)]
pub enum Confirm {
    Restart,
    MainMenu,
}

/// Which menu screen is open
pub enum Menu {
    Closed,
    /// How far down the options are scrolled, in pixels
    Options { scroll: f32 },
    /// The choice of player whose keys to change
    Players,
    Keys(KeysScreen),
    Confirm(Confirm),
}

impl Menu {
    pub fn is_open(&self) -> bool {
        !matches!(self, Menu::Closed)
    }

    pub fn open(&mut self) {
        *self = Menu::Options { scroll: 0. };
    }

    /// Esc: go back one screen (or cancel a key waiting to be bound)
    fn back(&mut self) -> Option<MenuEvent> {
        match self {
            Menu::Closed => return None,
            Menu::Options { .. } => {
                *self = Menu::Closed;
                return Some(MenuEvent::Closed);
            }
            Menu::Players => self.open(),
            Menu::Keys(keys) if keys.listening.is_some() => keys.listening = None,
            Menu::Keys(_) => *self = Menu::Players,
            Menu::Confirm(_) => self.open(),
        }
        None
    }

    /// Handle a key press while the menus are open: Esc goes back, and a key
    /// waiting to be bound takes any other press, except Space (play/pause)
    /// and keys that aren't recognised. Returns whether the press was used,
    /// and what the host should know about.
    pub fn key_pressed(&mut self, press: KeyPress, prefs: &mut Prefs) -> (bool, Option<MenuEvent>) {
        if !self.is_open() {
            return (false, None);
        }
        if press.code == KeyCode::Escape {
            return (true, self.back());
        }
        if let Menu::Keys(KeysScreen { listening: Some(_), .. }) = self {
            if !matches!(press.code, KeyCode::Space | KeyCode::Unknown) {
                self.bind_key(press.key, prefs);
            }
            return (true, None);
        }
        (false, None)
    }

    /// Bind `key` to the listening key of the keys screen. A key already bound
    /// elsewhere (for either player) moves here, and its old place flashes.
    fn bind_key(&mut self, key: Key, prefs: &mut Prefs) {
        let Menu::Keys(keys) = self else {
            return;
        };
        let Some(dir) = keys.listening.take() else {
            return;
        };

        for side in [Side::Left, Side::Right] {
            let controls = prefs.controls_mut(side);
            while let Some(old) = controls.dir_of(key) {
                controls.set(old, None);
                if (side, old) != (keys.side, dir) {
                    keys.flash(side, old);
                }
            }
        }
        prefs.controls_mut(keys.side).set(dir, Some(key));
        prefs.save();
    }

    /// Draw the open menu screen and act on a click. `options` are the
    /// options menu's toggles with their labels, a line (of one or more) at a
    /// time, top to bottom; `in_game`
    /// adds Restart and Main menu below them. The caller must have set the
    /// default (screen-space) camera.
    pub fn draw(&mut self, options: &[Vec<(Toggle, String)>], in_game: bool, prefs: &mut Prefs) -> Option<MenuEvent> {
        match self {
            Menu::Closed => {}
            Menu::Options { scroll } => {
                // buttons in reading order, and how they're laid out
                let mut entries = vec![(Entry::Close, "Close".to_string())];
                let mut lines = vec![Line::Buttons(vec!["Close".to_string()])];
                if in_game {
                    entries.push((Entry::Confirm(Confirm::Restart), "Restart".to_string()));
                    entries.push((Entry::Confirm(Confirm::MainMenu), "Main menu".to_string()));
                    lines.push(Line::Buttons(vec!["Restart".to_string(), "Main menu".to_string()]));
                }
                entries.push((Entry::Controls, "Controls".to_string()));
                lines.push(Line::Buttons(vec!["Controls".to_string()]));
                lines.push(Line::Gap);
                for line in options {
                    entries.extend(line.iter().map(|(toggle, label)| (Entry::Toggle(*toggle), label.clone())));
                    lines.push(Line::Buttons(line.iter().map(|(_, label)| label.clone()).collect()));
                }

                match options_menu::draw(&lines, scroll).map(|i| entries[i].0) {
                    Some(Entry::Close) => {
                        *self = Menu::Closed;
                        return Some(MenuEvent::Closed);
                    }
                    Some(Entry::Toggle(toggle)) => {
                        toggle.apply(prefs);
                        return Some(MenuEvent::Toggled(toggle));
                    }
                    Some(Entry::Controls) => *self = Menu::Players,
                    Some(Entry::Confirm(confirm)) => *self = Menu::Confirm(confirm),
                    None => {}
                }
            }
            Menu::Players => match controls_menu::draw_players(prefs.single_player) {
                Some(PlayersAction::Back) => self.open(),
                Some(PlayersAction::Open(side)) => *self = Menu::Keys(KeysScreen::new(side)),
                Some(PlayersAction::SinglePlayer(side)) => {
                    prefs.single_player = side;
                    prefs.save();
                }
                None => {}
            },
            Menu::Keys(keys) => match controls_menu::draw_keys(keys, prefs.controls(keys.side)) {
                Some(KeysAction::Back) => *self = Menu::Players,
                // clicking the key that's listening stops it
                Some(KeysAction::Select(dir)) => {
                    keys.listening = if keys.listening == Some(dir) { None } else { Some(dir) };
                }
                None => {}
            },
            Menu::Confirm(confirm) => {
                let question = match confirm {
                    Confirm::Restart => "Restart the game?",
                    Confirm::MainMenu => "Exit to main menu?",
                };
                match options_menu::draw_confirm(question) {
                    Some(true) => {
                        let event = match confirm {
                            Confirm::Restart => MenuEvent::Restart,
                            Confirm::MainMenu => MenuEvent::MainMenu,
                        };
                        *self = Menu::Closed;
                        return Some(event);
                    }
                    Some(false) => self.open(),
                    None => {}
                }
            }
        }
        None
    }
}

/// A button of the options menu
#[derive(Copy, Clone)]
enum Entry {
    Close,
    Toggle(Toggle),
    Controls,
    Confirm(Confirm),
}
