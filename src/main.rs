#![deny(unused_must_use)]

use enum_map_lite::enum_map;
use macroquad::input::get_keys_released;
use macroquad::window::{next_frame, screen_height, screen_width, Conf};

use crate::app::key::KeyInput;
use crate::app::screen::{Screen, StartScreen, Transition};
use crate::app::Palette;
use crate::apple::spawn::SpawnPolicy;
use crate::basic::{CellDim, Side};
use crate::snake::eat_mechanics::{EatBehavior, EatMechanics};
use crate::snake::SegmentType;
use crate::snake_control::appetite::Appetite;
use crate::snake_control::pathfinder;

#[macro_use]
mod support;
#[macro_use]
mod basic;
mod app;
mod button;
mod color;
mod snake;
mod view;
#[macro_use]
mod apple;
mod rendering;
pub mod snake_control;

/// On wasm32-unknown-unknown there is no default `getrandom` backend; this is
/// the custom one `.cargo/config.toml` selects there, backed by macroquad's
/// PRNG, so `rand` works without wasm-bindgen. Native targets use the OS.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
unsafe extern "Rust" fn __getrandom_v03_custom(dest: *mut u8, len: usize) -> Result<(), getrandom::Error> {
    // SAFETY: getrandom hands over `len` writable bytes at `dest` (possibly
    // uninitialized, hence written before the slice is made)
    let buf = unsafe {
        std::ptr::write_bytes(dest, 0, len);
        std::slice::from_raw_parts_mut(dest, len)
    };
    for chunk in buf.chunks_mut(4) {
        let bytes = macroquad::rand::rand().to_le_bytes();
        chunk.copy_from_slice(&bytes[..chunk.len()]);
    }
    Ok(())
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Hex Snake".to_owned(),
        window_width: 1200,
        window_height: 900,
        high_dpi: true,
        // without MSAA every polygon edge snaps to whole pixels, so a slowly
        // moving curved edge crawls instead of sliding — very visible on the
        // snake's round head/tail caps
        sample_count: 4,
        ..Default::default()
    }
}

/// Build a single keyboard-controlled player snake (mirrors the seed that
/// `App::new` used to construct).
/// The game mode's starvation is set when the game starts.
fn player_seed(side: Side) -> snake::builder::Builder {
    let eat_mechanics = EatMechanics::new(
        enum_map! {
            SegmentType::Eaten { .. } => EatBehavior::PassOver,
            _ => EatBehavior::Crash,
        },
        enum_map! {
            snake::Type::Rain => enum_map! { _ => EatBehavior::PassUnder },
            _ => enum_map! { _ => EatBehavior::Crash },
        },
    )
    .mark_passable();

    snake::builder::Builder::default()
        .snake_type(snake::Type::Player)
        .eat_mechanics(eat_mechanics)
        .palette(snake::PaletteTemplate::rainbow())
        .controller(snake_control::Template::Keyboard { side: Some(side) })
        .speed(5.)
        .autopilot(pathfinder::Template::WithBackup {
            main: Box::new(pathfinder::Template::WeightedBFS(pathfinder::Weights {
                teleport: 0,
                ..Default::default()
            })),
            backup: Box::new(pathfinder::Template::SpaceFilling),
        })
        // plan a route through the next three apples, not just the nearest one
        .autopilot_targets(3)
        // and go around the ones that would shrink it
        .autopilot_appetite(Appetite {
            shrink: -15,
            ..Default::default()
        })
}

#[macroquad::main(window_conf)]
async fn main() {
    // macroquad's PRNG starts from a fixed state, and on wasm it's also what
    // backs `rand::rng()` (see `__getrandom_v03_custom`), so every page load would
    // replay the same apples. Seed it from the wall clock (`get_time` counts
    // from startup, so it wouldn't vary).
    macroquad::rand::srand(macroquad::miniquad::date::now().to_bits());

    let cell_dim = CellDim::from(50.);
    // one player per side of the keyboard, in screen order
    let seeds = vec![player_seed(Side::Left), player_seed(Side::Right)];

    let mut screens: Vec<Box<dyn Screen>> = vec![Box::new(StartScreen::new(
        cell_dim,
        seeds,
        Palette::dark(),
        SpawnPolicy::Random { apple_count: 3 },
    ))];

    let mut last_size = (screen_width(), screen_height());
    let mut key_input = KeyInput::new();

    loop {
        let screen = screens.last_mut().expect("there is always a screen");

        let size = (screen_width(), screen_height());
        if size != last_size {
            last_size = size;
            let _ = screen.resize_event(size.0, size.1);
        }

        for key in key_input.poll() {
            let _ = screen.key_down_event(key);
        }
        for key in get_keys_released() {
            let _ = screen.key_up_event(key);
        }

        let _ = screen.update();
        if let Err(e) = screen.draw() {
            eprintln!("{e:?}");
        }
        match screen.transition() {
            Some(Transition::Push(next)) => screens.push(next),
            // the bottom screen (the start screen) never closes
            Some(Transition::Pop) if screens.len() > 1 => {
                screens.pop();
                if let Some(screen) = screens.last_mut() {
                    screen.resume();
                }
            }
            _ => {}
        }

        next_frame().await;
    }
}
