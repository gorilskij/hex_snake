#![feature(stmt_expr_attributes)]
#![feature(try_blocks)]
#![feature(exhaustive_patterns)]
#![feature(if_let_guard)]
#![deny(unused_must_use)]

#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate lazy_static;
extern crate core;

use macroquad::input::{get_keys_pressed, get_keys_released};
use macroquad::window::{next_frame, screen_height, screen_width, Conf};

use crate::app::keyboard_control::ControlSetup;
use crate::app::screen::Game;
use crate::app::Palette;
use crate::basic::{CellDim, Side};
use crate::gfx::event::EventHandler;
use crate::keyboard_layout::Layout;
use crate::snake::eat_mechanics::{EatBehavior, EatMechanics, Knowledge};
use crate::snake::SegmentType;
use crate::snake_control::pathfinder;

#[macro_use]
mod support;
#[macro_use]
mod basic;
mod app;
mod color;
mod gfx;
mod keyboard_layout;
mod snake;
mod view;
#[macro_use]
mod apple;
mod rendering;
pub mod snake_control;

// On wasm32-unknown-unknown there is no default `getrandom` backend; supply one
// backed by macroquad's PRNG so `rand`/`thread_rng` work without wasm-bindgen.
// On native targets the OS backend is used and this is ignored.
getrandom::register_custom_getrandom!(macroquad_getrandom);

#[allow(dead_code)]
fn macroquad_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    for chunk in buf.chunks_mut(4) {
        let bytes = macroquad::rand::rand().to_le_bytes();
        for (b, rb) in chunk.iter_mut().zip(bytes.iter()) {
            *b = *rb;
        }
    }
    Ok(())
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Hex Snake".to_owned(),
        window_width: 1200,
        window_height: 900,
        high_dpi: true,
        ..Default::default()
    }
}

/// Build a single keyboard-controlled player snake (mirrors the seed that
/// `App::new` used to construct).
fn player_seed(control_setup: ControlSetup) -> snake::builder::Builder {
    let eat_mechanics = EatMechanics::new(
        by_segment_type! {
            SegmentType::DISCR_EATEN => EatBehavior::PassOver,
            _ => EatBehavior::Crash,
        },
        by_snake_type! {
            snake::Type::Rain => by_segment_type! {
                _ => EatBehavior::PassUnder,
            },
            _ => by_segment_type! {
                _ => EatBehavior::Crash,
            },
        },
    );

    let knowledge = Knowledge::accurate(&eat_mechanics);

    snake::builder::Builder::default()
        .snake_type(snake::Type::Player)
        .eat_mechanics(eat_mechanics)
        .palette(snake::PaletteTemplate::rainbow(true))
        .controller(snake_control::Template::Keyboard { control_setup, knowledge })
        .speed(1.)
        .autopilot(pathfinder::Template::WithBackup {
            main: Box::new(pathfinder::Template::WeightedBFS),
            backup: Box::new(pathfinder::Template::SpaceFilling),
        })
}

#[macroquad::main(window_conf)]
async fn main() {
    let control_setup = ControlSetup {
        // web delivers physical (qwerty-position) key codes, so use the qwerty
        // bindings directly: ul=J u=K ur=L dl=M d=Comma dr=Period
        layout: Layout::Qwerty,
        keyboard_side: Side::Right,
        hand: Side::Right,
    };

    let cell_dim = CellDim::from(50.);
    let seeds = vec![player_seed(control_setup)];

    let mut game = Game::new(
        cell_dim,
        7.,
        seeds,
        Palette::dark(),
        crate::apple::spawn::SpawnPolicy::Random { apple_count: 5 },
    );

    let mut last_size = (screen_width(), screen_height());

    loop {
        let size = (screen_width(), screen_height());
        if size != last_size {
            last_size = size;
            let _ = game.resize_event(size.0, size.1);
        }

        for key in get_keys_pressed() {
            let _ = game.key_down_event(key);
        }
        for key in get_keys_released() {
            let _ = game.key_up_event(key);
        }

        let _ = game.update();
        if let Err(e) = game.draw() {
            eprintln!("{e:?}");
        }

        next_frame().await;
    }
}
