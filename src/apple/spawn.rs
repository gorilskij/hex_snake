use crate::{
    app::{game_context::GameContext, screen::Prefs},
    apple::{self, Apple},
    basic::{
        board::{get_occupied_cells, random_free_spot},
        HexPoint,
    },
    snake,
    snake::{EatBehavior, EatMechanics, PassthroughKnowledge, Snake},
    snake_control,
};
use rand::Rng;

#[allow(unused_macros)]
#[macro_export]
macro_rules! spawn_schedule {
    (@ spawn($h:expr, $v:expr) ) => {
        crate::apple::spawn::ScheduledSpawn::Spawn(
            crate::basic::HexPoint { h: $h, v: $v }
        )
    };
    (@ wait($t:expr) ) => {
        crate::apple::spawn::ScheduledSpawn::Wait {
            total: $t,
            current: 0,
        }
    };
    [ $( $action:tt ( $( $inner:tt )* ) ),* $(,)? ] => {
        vec![
            $(
                spawn_schedule!(@ $action( $( $inner )* ))
            ),*
        ]
    };
}

pub enum ScheduledSpawn {
    Spawn(HexPoint),
    Wait { total: usize, current: usize },
}

pub enum SpawnPolicy {
    None, // no apples
    Random {
        apple_count: usize,
    },
    // a new apple is spawed each time there are not enough apples on the board
    ScheduledOnEat {
        apple_count: usize,
        spawns: Vec<ScheduledSpawn>,
        next_index: usize,
    },
    // apples are spawned at a given time
    // ScheduledOtTime { .. }
}

impl SpawnPolicy {
    pub fn reset(&mut self) {
        match self {
            SpawnPolicy::None => {}
            SpawnPolicy::Random { .. } => {}
            SpawnPolicy::ScheduledOnEat { next_index, .. } => *next_index = 0,
        }
    }
}

macro_rules! float_sum {
    () => { 0. };
    ($x:expr) => { $x };
    ($x:expr, $( $rest:tt )*) => { $x + float_sum!($( $rest )*) };
}

// TODO: include a sum-to-<1 check
// randomly choose one of a number of options with a given probability each
// and with a catch-all option
macro_rules! choose {
    ($rand:ident ;; $( $probs:expr ),* ;; $prob:expr => $then:expr, $( $rest:tt )*) => {
        if $rand < float_sum!($( $probs ),*, $prob) {
            $then
        } else {
            choose!($rand ;; $( $probs ),*, $prob ;; $( $rest )*)
        }
    };
    ($_rand:ident ;; $( $_probs:expr ),* ;; $otherwise:expr $( , )?) => {
        {
            $otherwise
        }
    };
    (let $rand:ident: f64 <- $rng:expr; $( $tokens:tt )*) => {
        {
            let $rand = $rng.gen::<f64>();
            choose!($rand ;; 0. ;; $( $tokens )*)
        }
    };
}

// TODO: add a snake spawn policy
// TODO: factor ai snake palettes out into game palette
fn generate_apple_type(prefs: &Prefs, rng: &mut impl Rng) -> apple::Type {
    if prefs.special_apples {
        choose! {
            let rand: f64 <- rng;
            prefs.prob_spawn_competitor => {
                apple::Type::SpawnSnake(Box::new(snake::Builder::default()
                        .snake_type(snake::Type::Competitor { life: Some(200) })
                        .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                        .palette(snake::PaletteTemplate::pastel_rainbow(true))
                        .controller(snake_control::Template::AStar { passthrough_knowledge: PassthroughKnowledge::always(false) })
                        .speed(1.)
                ))
            },
            prefs.prob_spawn_killer => {
                apple::Type::SpawnSnake(Box::new(snake::Builder::default()
                    .snake_type(snake::Type::Killer { life: Some(200) })
                    .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                    .palette(snake::PaletteTemplate::dark_blue_to_red(false))
                    // .palette(snake::PaletteTemplate::dark_rainbow(true))
                    .controller(snake_control::Template::Killer)
                    .speed(1.)
                ))
            },
            prefs.prob_spawn_rain => {
                apple::Type::SpawnRain
            },
            apple::Type::Food(prefs.apple_food),
        }
    } else {
        apple::Type::Food(prefs.apple_food)
    }
}

pub fn spawn_apples(
    snakes: &[Snake],
    apples: &[Apple],
    gtx: &mut GameContext,
    rng: &mut impl Rng,
) -> Vec<Apple> {
    // lazy
    let mut occupied_cells = None;

    let mut spawn = vec![];

    loop {
        let can_spawn = match &gtx.apple_spawn_policy {
            SpawnPolicy::None => false,
            SpawnPolicy::Random { apple_count } => apples.len() + spawn.len() < *apple_count,
            SpawnPolicy::ScheduledOnEat { apple_count, .. } => {
                apples.len() + spawn.len() < *apple_count
            }
        };

        if !can_spawn {
            break;
        }

        let occupied_cells =
            occupied_cells.get_or_insert_with(|| get_occupied_cells(snakes, apples));

        let new_apple_pos = match &mut gtx.apple_spawn_policy {
            SpawnPolicy::None => panic!("shouldn't be spawning with SpawnPolicy::None"),
            SpawnPolicy::Random { apple_count } => {
                let apple_pos = match random_free_spot(occupied_cells, gtx.board_dim, rng) {
                    Some(pos) => pos,
                    None => {
                        println!(
                            "warning: no space left for new apples ({} apples will be missing)",
                            *apple_count - apples.len(),
                        );
                        break;
                    }
                };

                // insert at sorted position
                match occupied_cells.binary_search(&apple_pos) {
                    Ok(_) => panic!("Spawned apple on top of another apple at {:?}", apple_pos),
                    Err(idx) => occupied_cells.insert(idx, apple_pos),
                }

                Some(apple_pos)
            }
            SpawnPolicy::ScheduledOnEat { spawns, next_index, .. } => {
                let len = spawns.len();
                match &mut spawns[*next_index] {
                    ScheduledSpawn::Wait { total, current } => {
                        if *current == *total - 1 {
                            *current = 0;
                            *next_index = (*next_index + 1) % len;
                        } else {
                            *current += 1;
                        }
                        None
                    }
                    ScheduledSpawn::Spawn(pos) => {
                        *next_index = (*next_index + 1) % len;
                        Some(*pos)
                    }
                }
            }
        };

        match new_apple_pos {
            Some(pos) => spawn.push(Apple {
                pos,
                apple_type: generate_apple_type(&gtx.prefs, rng),
            }),
            None => break,
        }
    }

    spawn
}
