use crate::{
    app::{
        apple::{self, Apple},
        screen::Prefs,
        snake,
        snake::{controller::Template, EatBehavior, EatMechanics, Snake},
        utils::{get_occupied_cells, random_free_spot},
    },
    basic::{HexDim, HexPoint},
};
use rand::Rng;

#[allow(unused_macros)]
macro_rules! spawn_schedule {
    (@ spawn($h:expr, $v:expr) ) => {
        crate::app::apple::spawn::ScheduledSpawn::Spawn(
            crate::basic::HexPoint { h: $h, v: $v }
        )
    };
    (@ wait($t:expr) ) => {
        crate::app::apple::spawn::ScheduledSpawn::Wait {
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

// TODO: add a snake spawn policy
fn generate_apple_type(prefs: &Prefs, rng: &mut impl Rng) -> apple::Type {
    if prefs.special_apples {
        let rand = rng.gen::<f32>();
        if rand < prefs.prob_spawn_competitor {
            apple::Type::SpawnSnake(snake::Seed {
                snake_type: snake::Type::Competitor { life: Some(200) },
                eat_mechanics: EatMechanics::always(EatBehavior::Die),
                palette: snake::PaletteTemplate::pastel_rainbow(true),
                controller: Template::AStar,
                pos: None,
                dir: None,
                len: None,
            })
        } else if rand < prefs.prob_spawn_competitor + prefs.prob_spawn_killer {
            apple::Type::SpawnSnake(snake::Seed {
                snake_type: snake::Type::Killer { life: Some(200) },
                eat_mechanics: EatMechanics::always(EatBehavior::Die),
                palette: snake::PaletteTemplate::dark_blue_to_red(false),
                controller: Template::Killer,
                pos: None,
                dir: None,
                len: None,
            })
        } else {
            apple::Type::Normal(prefs.apple_food)
        }
    } else {
        apple::Type::Normal(prefs.apple_food)
    }
}

pub fn spawn_apples(
    policy: &mut SpawnPolicy,
    board_dim: HexDim,
    snakes: &[Snake],
    apples: &[Apple],
    prefs: &Prefs,
    rng: &mut impl Rng,
) -> Vec<Apple> {
    // lazy
    let mut occupied_cells = None;

    let mut spawn = vec![];

    loop {
        let can_spawn = match policy {
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

        let new_apple_pos = match policy {
            SpawnPolicy::None => panic!("shouldn't be spawning with SpawnPolicy::None"),
            SpawnPolicy::Random { apple_count } => {
                let apple_pos = match random_free_spot(occupied_cells, board_dim, rng) {
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
                apple_type: generate_apple_type(prefs, rng),
            }),
            None => break,
        }
    }

    spawn
}
