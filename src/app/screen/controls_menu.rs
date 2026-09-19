//! The controls menus, drawn over the paused game like the options menu:
//! a choice between the two players (and whose keys a single player uses),
//! and each player's keys, laid out as a hexagon with a key on each corner.
//!
//! Immediate-mode, like the options menu: everything is rebuilt each frame
//! from the current bindings.

use macroquad::color::Color;
use macroquad::shapes::draw_line;
use macroquad::time::get_time;
use macroquad::window::{screen_height, screen_width};

use super::options_menu::{draw_overlay, wide_button};
use crate::app::key::{Controls, Key};
use crate::basic::{CellDim, Dir, Point, Side};
use crate::button::style::{ACCENT, BUTTON_CELL_DIM, BUTTON_COLOR, FONT_SIZE, STROKE_THICKNESS};
use crate::button::{Button, ButtonData, TriColor};
use crate::rendering::shape::{Hexagon, Shape, ShapePoints};
use crate::support::text::{draw_text, measure_text};

/// Share of the window's width each player button takes
const PLAYER_BUTTON_WIDTH: f32 = 0.35;
/// Distance of the big hexagon's corners from its center, relative to the
/// smaller of the window's dimensions, and at most
const KEYS_RADIUS: f32 = 0.2;
const MAX_KEYS_RADIUS: f32 = 140.;
/// Size of a key, relative to the big hexagon (radius to radius)
const KEY_RADIUS: f32 = 0.4;
/// Size of a key's label relative to the key's radius, and of the words in a
/// label (Ctrl, Shift, …) relative to that
const KEY_FONT: f32 = 0.68;
const KEY_WORD_FONT: f32 = 0.5;
/// How long a key that lost its binding blinks, and how fast
const FLASH_SECS: f64 = 0.9;
const FLASH_HZ: f64 = 5.;

const LISTENING_COLOR: TriColor = uniform(ACCENT);
const FLASH_COLOR: TriColor = uniform(Color::new(1., 0.2, 0.2, 1.));
const EMPTY_COLOR: TriColor = TriColor {
    normal: Color::new(0.3, 0.3, 0.3, 1.),
    ..BUTTON_COLOR
};

const fn uniform(color: Color) -> TriColor {
    TriColor { normal: color, hover: color, pressed: color }
}

fn side_name(side: Side) -> &'static str {
    match side {
        Side::Left => "Left player",
        Side::Right => "Right player",
    }
}

/// A back button centered at the bottom of the window
fn back_button() -> bool {
    let data = wide_button(BUTTON_CELL_DIM.side * 5., "Back");
    let mut button = Button::click(Point::zero(), data);
    let size = button.size();
    button.pos = Point {
        x: (screen_width() - size.x) / 2.,
        y: screen_height() - BUTTON_CELL_DIM.side - size.y,
    };
    button.draw()
}

pub enum PlayersAction {
    Back,
    /// Open this player's keys
    Open(Side),
    /// Make this player's keys the single player's
    SinglePlayer(Side),
}

/// The two players side by side, each with a radio button underneath for
/// whose keys a single player uses.
pub fn draw_players(single_player: Side) -> Option<PlayersAction> {
    draw_overlay();
    let (width, height) = (screen_width(), screen_height());

    let button_width = PLAYER_BUTTON_WIDTH * width;
    let button_height = BUTTON_CELL_DIM.height();
    let gap = (width - 2. * button_width) / 3.;
    let y = height / 2. - button_height;

    let radio_dim = BUTTON_CELL_DIM;
    let radio = Hexagon::new(radio_dim);
    let dot_dim = radio_dim * 0.55;
    let dot = Hexagon::new(dot_dim);
    let dot_offset = Hexagon::center(radio_dim) - Hexagon::center(dot_dim);
    let radio_y = y + button_height * 1.75;

    let mut action = None;
    for (i, side) in [Side::Left, Side::Right].into_iter().enumerate() {
        let x = gap + i as f32 * (button_width + gap);
        if Button::click(Point { x, y }, wide_button(button_width, side_name(side))).draw() {
            action = Some(PlayersAction::Open(side));
        }

        let mut data = ButtonData::new(radio.clone(), STROKE_THICKNESS, BUTTON_COLOR);
        if side == single_player {
            data = data.inner_fill(dot.clone(), dot_offset, BUTTON_COLOR);
        }
        let radio_x = x + (button_width - radio_dim.width()) / 2.;
        if Button::click(Point { x: radio_x, y: radio_y }, data).draw() {
            action = Some(PlayersAction::SinglePlayer(side));
        }

        if side == Side::Left {
            let label = "Single player:";
            let dims = measure_text(label, FONT_SIZE);
            let baseline = radio_y + (radio_dim.height() - dims.height) / 2. + dims.offset_y;
            let label_x = radio_x - BUTTON_CELL_DIM.side - dims.width;
            draw_text(label, label_x, baseline, FONT_SIZE, BUTTON_COLOR.normal);
        }
    }

    if back_button() {
        action = Some(PlayersAction::Back);
    }
    action
}

/// The state of one player's keys screen
pub struct KeysScreen {
    pub side: Side,
    /// The key waiting for a key press to bind
    pub listening: Option<Dir>,
    /// A key that just lost its binding to another, and since when (seconds)
    pub flash: Option<(Side, Dir, f64)>,
}

impl KeysScreen {
    pub fn new(side: Side) -> Self {
        Self { side, listening: None, flash: None }
    }

    pub fn flash(&mut self, side: Side, dir: Dir) {
        self.flash = Some((side, dir, get_time()));
    }

    fn flashing(&self, dir: Dir) -> bool {
        let Some((side, flash_dir, start)) = self.flash else {
            return false;
        };
        let t = get_time() - start;
        side == self.side && flash_dir == dir && t < FLASH_SECS && (t * FLASH_HZ).fract() < 0.5
    }
}

pub enum KeysAction {
    Back,
    /// A key was clicked
    Select(Dir),
}

/// One player's keys: a big hexagon, pointy at the top and bottom, with the
/// key for each direction on the corner that points that way.
pub fn draw_keys(screen: &KeysScreen, controls: &Controls) -> Option<KeysAction> {
    draw_overlay();
    let (width, height) = (screen_width(), screen_height());

    let title = side_name(screen.side);
    let dims = measure_text(title, FONT_SIZE);
    let title_y = BUTTON_CELL_DIM.side + dims.offset_y;
    draw_text(title, (width - dims.width) / 2., title_y, FONT_SIZE, BUTTON_COLOR.normal);

    let hint = if screen.listening.is_some() {
        "Press a key (Esc to cancel)"
    } else {
        "Click a key to change it"
    };
    let hint_size = FONT_SIZE * 0.6;
    let dims = measure_text(hint, hint_size);
    let hint_y = title_y + FONT_SIZE;
    draw_text(hint, (width - dims.width) / 2., hint_y, hint_size, BUTTON_COLOR.normal);

    let center = Point { x: width / 2., y: height / 2. };
    let radius = (KEYS_RADIUS * width.min(height)).min(MAX_KEYS_RADIUS);
    // clockwise from the top, like `Dir`
    let corner = |i: usize| {
        let angle = (-90. + 60. * i as f32).to_radians();
        center + Point { x: angle.cos(), y: angle.sin() } * radius
    };

    // a flat-topped hexagon's corners are one side from its center
    let key_dim = CellDim::from(KEY_RADIUS * radius);

    // The big hexagon's sides, stopping short of the keys. Every side leaves
    // its corner at 30° off the horizontal, straight through the middle of
    // one of the key's flat sides, so it has half the key's height to clear.
    let clearance = key_dim.height() / 2. + 2. * STROKE_THICKNESS;
    for i in 0..6 {
        let (a, b) = (corner(i), corner((i + 1) % 6));
        let along = (b - a) / radius;
        let (a, b) = (a + along * clearance, b - along * clearance);
        draw_line(a.x, a.y, b.x, b.y, STROKE_THICKNESS, BUTTON_COLOR.normal);
    }

    let key_shape: ShapePoints = Hexagon::new(key_dim);
    let key_center = Hexagon::center(key_dim);
    let font_size = KEY_FONT * key_dim.side;

    let mut action = None;
    for i in 0..6 {
        let dir = Dir::from(i as u8);
        let key = controls.get(dir);
        let color = if screen.listening == Some(dir) {
            LISTENING_COLOR
        } else if screen.flashing(dir) {
            FLASH_COLOR
        } else if key.is_none() {
            EMPTY_COLOR
        } else {
            BUTTON_COLOR
        };

        let mut data = ButtonData::new(key_shape.clone(), STROKE_THICKNESS, color);
        if let Some(key) = key {
            data = data.spans(label_spans(key, font_size), color);
        }
        if Button::click(corner(i) - key_center, data).draw() {
            action = Some(KeysAction::Select(dir));
        }
    }

    if back_button() {
        action = Some(KeysAction::Back);
    }
    action
}

/// A key's label in pieces: `L`/`R` full size, a word smaller
fn label_spans(key: Key, font_size: f32) -> Vec<(String, f32)> {
    let label = key.label();
    let name_size = if label.small { KEY_WORD_FONT * font_size } else { font_size };
    label
        .side
        .map(|side| (side.to_string(), font_size))
        .into_iter()
        .chain([(label.name, name_size)])
        .collect()
}
