//! A headless benchmark of the world update with many snakes, to see what
//! planning costs. Run it in release, on demand:
//!
//! `cargo test --release benchmark -- --ignored --nocapture`

use std::time::{Duration, Instant};

use rand::thread_rng;

use crate::app::game_context::GameContext;
use crate::app::game_mode::GameMode;
use crate::app::prefs::Prefs;
use crate::app::screen::Environment;
use crate::app::snake_management::{
    advance_snakes, find_collisions, handle_apple_collisions, handle_snake_collisions, relocate_covered_apples,
    spawn_snakes, update_snake_dirs,
};
use crate::apple::spawn::{expire_apples, spawn_apples, spawn_bad_apples, SpawnPolicy};
use crate::basic::{CellDim, HexDim};
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::eat_mechanics::{EatBehavior, EatMechanics};
use crate::snake::{self, PaletteTemplate};
use crate::snake_control::pathfinder;
use crate::{app, snake_control};

const BOARD: HexDim = HexDim { h: 80, v: 50 };
const SNAKES: usize = 24;
const APPLES: usize = 80;
/// A 60 Hz frame, one tick per frame (the snakes are slow enough for that).
const TICK: Duration = Duration::from_micros(16_667);
const TICKS: usize = 6_000;

fn competitor(targets: usize) -> SnakeBuilder {
    SnakeBuilder::default()
        .snake_type(snake::Type::Competitor { life: None })
        .eat_mechanics(EatMechanics::always(EatBehavior::Die))
        .palette(PaletteTemplate::pastel_rainbow())
        .controller(snake_control::Template::AppleSeeker {
            pathfinder: pathfinder::Template::WeightedBFS(Default::default()),
            targets,
            appetite: Default::default(),
        })
        .speed(5.)
}

#[derive(Default)]
struct Timings {
    planning: Duration,
    /// The slowest tick's planning, and which tick it was
    worst_planning: (Duration, usize),
    rest: Duration,
    deaths: usize,
}

/// One game's worth of ticks, as `Game::update` runs them, minus drawing.
fn run(targets: usize) -> Timings {
    // plain apples only: no snakes spawning, killers need a player to chase
    let mut prefs = Prefs::default();
    prefs.special_apples = false;
    let mut env = Environment {
        snakes: vec![],
        apples: vec![],
        portals: vec![],
        gtx: GameContext::new(
            BOARD,
            CellDim::from(10.),
            app::Palette::dark(),
            prefs,
            SpawnPolicy::Random { apple_count: APPLES },
            GameMode::Classic,
        ),
        rng: thread_rng(),
    };
    let seed = competitor(targets);
    spawn_snakes(&mut env, vec![seed.clone(); SNAKES]).unwrap();
    spawn_apples(&mut env);

    let mut timings = Timings::default();
    for tick in 0..TICKS {
        let start = Instant::now();
        let new_cells = advance_snakes(&mut env, TICK);
        let collisions = find_collisions(&env);
        let _ = handle_snake_collisions(&mut env, &collisions);
        let seeds = handle_apple_collisions(&mut env, &collisions);
        relocate_covered_apples(&mut env);
        expire_apples(&mut env, TICK);
        spawn_bad_apples(&mut env, TICK);
        spawn_snakes(&mut env, seeds).unwrap();
        if new_cells {
            spawn_apples(&mut env);
        }
        // keep the board as busy as it started
        let missing = SNAKES.saturating_sub(env.snakes.len());
        timings.deaths += missing;
        spawn_snakes(&mut env, vec![seed.clone(); missing]).unwrap();
        timings.rest += start.elapsed();

        let start = Instant::now();
        update_snake_dirs(&mut env);
        let planning = start.elapsed();
        timings.planning += planning;
        if planning > timings.worst_planning.0 {
            timings.worst_planning = (planning, tick);
        }
    }
    timings
}

#[test]
#[ignore = "a benchmark: run it in release, with --ignored --nocapture"]
fn benchmark() {
    let per_tick = |total: Duration| total / TICKS as u32;
    println!(
        "{}x{} board, {SNAKES} snakes, {APPLES} apples, {TICKS} ticks of {TICK:?}",
        BOARD.h, BOARD.v
    );
    for targets in [1, 3] {
        let t = run(targets);
        println!(
            "{targets} target(s) each: planning {:?}/tick (worst {:?}, tick {}), rest {:?}/tick, {} deaths",
            per_tick(t.planning),
            t.worst_planning.0,
            t.worst_planning.1,
            per_tick(t.rest),
            t.deaths,
        );
    }
}
