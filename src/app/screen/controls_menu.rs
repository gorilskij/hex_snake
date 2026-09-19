//! The controls menu, drawn over the paused game like the options menu:
//! both players' keys side by side, each laid out as a hexagon with a key on
//! each corner and, in the middle, a radio button for whose keys a single
//! player uses.
//!
//! Immediate-mode, like the options menu: everything is rebuilt each frame
//! from the current bindings.

use macroquad::color::Color;
use macroquad::shapes::draw_line;
use macroquad::time::get_time;
use macroquad::window::{screen_height, screen_width};

use super::options_menu::{draw_overlay, wide_button};
use crate::app::key::{Controls, Key};
use crate::app::prefs::Prefs;
use crate::basic::{CellDim, Dir, Point, Side};
use crate::button::style::{ACCENT, BUTTON_CELL_DIM, BUTTON_COLOR, FONT_SIZE, STROKE_THICKNESS};
use crate::button::{Button, ButtonData, TriColor};
use crate::rendering::shape::{Hexagon, Shape, ShapePoints};
use crate::support::text::{draw_text, measure_text};

/// Distance of a big hexagon's corners from its center: at most this share
/// of the window's height, and at most
const KEYS_RADIUS: f32 = 0.2;
const MAX_KEYS_RADIUS: f32 = 140.;
/// Distance between the two big hexagons' centers, relative to their radius
const KEYS_SPACING: f32 = 3.6;
/// Size of a key, relative to the big hexagon (radius to radius)
const KEY_RADIUS: f32 = 0.4;
/// Size of the single-player radio button, relative to the big hexagon
const RADIO_RADIUS: f32 = 0.3;
/// Size of a key's label relative to the key's radius, and of the words in a
/// label (Ctrl, Shift, …) relative to that
const KEY_FONT: f32 = 0.68;
const KEY_WORD_FONT: f32 = 0.5;
/// Size of the explanatory text, relative to the buttons' font
const NOTE_FONT: f32 = 0.6;
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

/// Draw `text` centered on `x`, with its top at `top`
fn centered_text(text: &str, x: f32, top: f32, font_size: f32) {
    let dims = measure_text(text, font_size);
    draw_text(text, x - dims.width / 2., top + dims.offset_y, font_size, BUTTON_COLOR.normal);
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

/// The state of the controls screen
#[derive(Default)]
pub struct ControlsScreen {
    /// The key waiting for a key press to bind
    pub listening: Option<(Side, Dir)>,
    /// A key that just lost its binding to another, and since when (seconds)
    pub flash: Option<(Side, Dir, f64)>,
}

impl ControlsScreen {
    pub fn flash(&mut self, side: Side, dir: Dir) {
        self.flash = Some((side, dir, get_time()));
    }

    fn flashing(&self, side: Side, dir: Dir) -> bool {
        let Some((flash_side, flash_dir, start)) = self.flash else {
            return false;
        };
        let t = get_time() - start;
        (flash_side, flash_dir) == (side, dir) && t < FLASH_SECS && (t * FLASH_HZ).fract() < 0.5
    }
}

pub enum ControlsAction {
    Back,
    /// A key was clicked
    Select(Side, Dir),
    /// Make this player's keys the single player's
    SinglePlayer(Side),
}

/// Both players' keys, side by side.
pub fn draw_controls(screen: &ControlsScreen, prefs: &Prefs) -> Option<ControlsAction> {
    draw_overlay();
    let (width, height) = (screen_width(), screen_height());

    let hint = if screen.listening.is_some() {
        "Press a key (Esc to cancel)"
    } else {
        "Click a key to change it"
    };
    centered_text(hint, width / 2., BUTTON_CELL_DIM.side, FONT_SIZE * NOTE_FONT);

    // the hexagons, keys included, fit side by side in the window
    let outer = 1. + KEY_RADIUS;
    let radius = (KEYS_RADIUS * height)
        .min(width / (KEYS_SPACING + 2. * outer + 0.5))
        .min(MAX_KEYS_RADIUS);
    let reach = outer * radius + BUTTON_CELL_DIM.side / 2.;

    let mut action = None;
    for (i, side) in [Side::Left, Side::Right].into_iter().enumerate() {
        let center = Point {
            x: width / 2. + (i as f32 - 0.5) * KEYS_SPACING * radius,
            y: height / 2.,
        };
        centered_text(side_name(side), center.x, center.y - reach - FONT_SIZE, FONT_SIZE);

        if let Some(clicked) = draw_keys(screen, side, prefs.controls(side), center, radius) {
            action = Some(ControlsAction::Select(side, clicked));
        }
        if radio_button(center, RADIO_RADIUS * radius, side == prefs.single_player) {
            action = Some(ControlsAction::SinglePlayer(side));
        }
    }

    let note = "Center button selects which configuration to use for single player";
    centered_text(note, width / 2., height / 2. + reach, FONT_SIZE * NOTE_FONT);

    if back_button() {
        action = Some(ControlsAction::Back);
    }
    action
}

/// A hexagonal radio button of radius `radius` centered at `center`, filled
/// in when `selected`; returns whether it was clicked
fn radio_button(center: Point, radius: f32, selected: bool) -> bool {
    let dim = CellDim::from(radius);
    let mut data = ButtonData::new(Hexagon::new(dim), STROKE_THICKNESS, BUTTON_COLOR);
    if selected {
        let dot_dim = dim * 0.55;
        let offset = Hexagon::center(dim) - Hexagon::center(dot_dim);
        data = data.inner_fill(Hexagon::new(dot_dim), offset, BUTTON_COLOR);
    }
    Button::click(center - Hexagon::center(dim), data).draw()
}

/// One player's keys: a big hexagon, pointy at the top and bottom, with the
/// key for each direction on the corner that points that way. Returns the
/// direction whose key was clicked, if any.
fn draw_keys(screen: &ControlsScreen, side: Side, controls: &Controls, center: Point, radius: f32) -> Option<Dir> {
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

    let mut clicked = None;
    for i in 0..6 {
        let dir = Dir::from(i as u8);
        let key = controls.get(dir);
        let color = if screen.listening == Some((side, dir)) {
            LISTENING_COLOR
        } else if screen.flashing(side, dir) {
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
            clicked = Some(dir);
        }
    }
    clicked
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
