use std::time::Duration;

use rand::distributions::uniform::SampleRange;
use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;

use crate::app::game_context::GameContext;
use crate::app::game_mode::{hunger, GameMode};
use crate::app::screen::Environment;
use crate::apple::{self, Apple};
use crate::basic::board::{get_occupied_cells, occupied_or_near_players, random_free_spot};
use crate::basic::{Frames, HexPoint};
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::eat_mechanics::{EatBehavior, EatMechanics};
use crate::snake_control::pathfinder;
use crate::{snake, snake_control};

// #[allow(unused_macros)]
// #[macro_export]
// macro_rules! spawn_schedule {
//     (@ spawn($h:expr, $v:expr) ) => {
//         $crate::apple::spawn::ScheduledSpawn::Spawn(
//             $crate::basic::HexPoint { h: $h, v: $v }
//         )
//     };
//     (@ wait($t:expr) ) => {
//         $crate::apple::spawn::ScheduledSpawn::Wait {
//             total: $t,
//             current: 0,
//         }
//     };
//     [ $( $action:tt ( $( $inner:tt )* ) ),* $(,)? ] => {
//         vec![
//             $(
//                 spawn_schedule!(@ $action( $( $inner )* ))
//             ),*
//         ]
//     };
// }

#[derive(Clone)]
pub enum SpawnEvent {
    Spawn(Apple),
    Wait(Frames),
}

pub type SpawnSchedule = Vec<SpawnEvent>;

pub struct SpawnScheduleBuilder(SpawnSchedule);

impl SpawnScheduleBuilder {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn spawn(mut self, pos: HexPoint, apple_type: apple::Type) -> Self {
        self.0
            .push(SpawnEvent::Spawn(Apple { pos, apple_type, time_left: None }));
        self
    }

    pub fn wait(mut self, frames: Frames) -> Self {
        self.0.push(SpawnEvent::Wait(frames));
        self
    }

    pub fn build(self) -> SpawnSchedule {
        self.0
    }
}

// TODO: specify which types of apples spawn when
#[derive(Clone)]
pub enum SpawnPolicy {
    None, // no apples
    Random {
        apple_count: usize,
    },
    // a new apple is spawned each time there are not enough apples on the board
    ScheduledOnEat {
        apple_count: usize,
        schedule: Vec<SpawnEvent>,
        next_index: usize,
        current_wait: Frames,
    },
    // apples are spawned at a given time
    // ScheduledOtTime { .. }
}

impl SpawnPolicy {
    pub fn reset(&mut self) {
        match self {
            SpawnPolicy::None => {}
            SpawnPolicy::Random { .. } => {}
            SpawnPolicy::ScheduledOnEat { next_index, current_wait, .. } => {
                *next_index = 0;
                *current_wait = 0;
            }
        }
    }
}

/// The ordinary apple of the game mode.
pub fn food_apple(gtx: &GameContext) -> apple::Type {
    match gtx.mode {
        GameMode::Classic => apple::Type::Eat(gtx.prefs.apple_food),
        GameMode::Hunger => apple::Type::Grow(hunger::GROW),
    }
}

// TODO: add a snake spawn policy
fn generate_apple_type(gtx: &GameContext, rng: &mut impl Rng) -> apple::Type {
    let prefs = &gtx.prefs;
    let palette = &gtx.palette;
    if prefs.special_apples {
        let mut weights = [
            prefs.prob_spawn_competitor,
            prefs.prob_spawn_killer,
            prefs.prob_spawn_rain,
            0.0,
        ];
        weights[weights.len() - 1] = 1.0 - weights.iter().sum::<f64>();
        match WeightedIndex::new(weights).unwrap().sample(rng) {
            0 => apple::Type::SpawnSnake(Box::new(
                SnakeBuilder::default()
                    .snake_type(snake::Type::Competitor { life: Some(200) })
                    .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                    .palette(palette.palette_competitor)
                    .starvation(gtx.mode.starvation())
                    .controller(snake_control::Template::AppleSeeker(pathfinder::Template::WeightedBFS(
                        pathfinder::Weights { sharp_turn: 8, ..Default::default() },
                    )))
                    .speed(1.),
            )),
            1 => apple::Type::SpawnSnake(Box::new(
                SnakeBuilder::default()
                    .snake_type(snake::Type::Killer { life: Some(200) })
                    .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                    .palette(palette.palette_killer)
                    .controller(snake_control::Template::Killer)
                    .speed(1.),
            )),
            2 => apple::Type::SpawnRain,
            _ => food_apple(gtx),
        }
    } else {
        food_apple(gtx)
    }
}

pub fn spawn_apples<Rng: rand::Rng>(env: &mut Environment<Rng>) {
    // lazy
    let mut occupied_cells = None;

    let mut spawn = vec![];

    // apples that expire on their own don't count towards the policy
    let permanent = env.apples.iter().filter(|apple| apple.time_left.is_none()).count();

    loop {
        let can_spawn = match &env.gtx.apple_spawn_policy {
            SpawnPolicy::None => false,
            SpawnPolicy::Random { apple_count } => permanent + spawn.len() < *apple_count,
            SpawnPolicy::ScheduledOnEat { apple_count, .. } => permanent + spawn.len() < *apple_count,
        };

        if !can_spawn {
            break;
        }

        let occupied_cells = occupied_cells.get_or_insert_with(|| get_occupied_cells(&env.snakes, &env.apples));

        let new_apple = match &mut env.gtx.apple_spawn_policy {
            SpawnPolicy::None => panic!("shouldn't be spawning with SpawnPolicy::None"),
            SpawnPolicy::Random { apple_count } => {
                let apple_pos = match random_free_spot(occupied_cells, env.gtx.board_dim, &mut env.rng) {
                    Some(pos) => pos,
                    None => {
                        println!(
                            "warning: no space left for new apples ({} apples will be missing)",
                            *apple_count - env.apples.len(),
                        );
                        break;
                    }
                };

                // insert at sorted position
                match occupied_cells.binary_search(&apple_pos) {
                    Ok(_) => panic!("Spawned apple on top of another apple at {apple_pos:?}"),
                    Err(idx) => occupied_cells.insert(idx, apple_pos),
                }

                let apple_type = generate_apple_type(&env.gtx, &mut env.rng);
                Some(Apple {
                    pos: apple_pos,
                    apple_type,
                    time_left: None,
                })
            }
            SpawnPolicy::ScheduledOnEat {
                schedule: spawns,
                next_index,
                current_wait,
                ..
            } => {
                let len = spawns.len();
                match &mut spawns[*next_index] {
                    SpawnEvent::Wait(frames) => {
                        if *current_wait == *frames - 1 {
                            *current_wait = 0;
                            *next_index = (*next_index + 1) % len;
                        } else {
                            *current_wait += 1;
                        }
                        None
                    }
                    SpawnEvent::Spawn(pos) => {
                        *next_index = (*next_index + 1) % len;
                        Some(pos.clone())
                    }
                }
            }
        };

        match new_apple {
            Some(apple) => spawn.push(apple),
            None => break,
        }
    }

    env.apples.extend(spawn);
}

/// Count down the apples that expire and remove the ones whose time is up.
pub fn expire_apples<Rng>(env: &mut Environment<Rng>, elapsed: Duration) {
    env.apples.retain_mut(|apple| match &mut apple.time_left {
        None => true,
        Some(time_left) => {
            *time_left = time_left.saturating_sub(elapsed);
            !time_left.is_zero()
        }
    });
}

/// Hunger mode: now and then a bad apple appears, away from the players.
pub fn spawn_bad_apples<Rng: rand::Rng>(env: &mut Environment<Rng>, elapsed: Duration) {
    if env.gtx.mode != GameMode::Hunger {
        return;
    }

    let bad_apples = env
        .apples
        .iter()
        .filter(|apple| matches!(apple.apple_type, apple::Type::Shrink(_)))
        .count();
    if bad_apples >= hunger::MAX_BAD_APPLES {
        return;
    }

    // arrivals at random times, one per interval on average
    let chance = 1. - (-elapsed.as_secs_f64() / hunger::BAD_APPLE_INTERVAL as f64).exp();
    if !env.rng.gen_bool(chance) {
        return;
    }

    let blocked = occupied_or_near_players(&env.snakes, &env.apples, env.gtx.board_dim);
    let Some(pos) = random_free_spot(&blocked, env.gtx.board_dim, &mut env.rng) else {
        return;
    };
    let amount = hunger::BAD_APPLE_SHRINK.sample_single(&mut env.rng);
    env.apples.push(Apple {
        pos,
        apple_type: apple::Type::Shrink(amount),
        time_left: Some(hunger::BAD_APPLE_LIFETIME),
    });
}
