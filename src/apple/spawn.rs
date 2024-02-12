use crate::app::screen::{Environment, Prefs};
use crate::apple::{self, Apple};
use crate::basic::board::{get_occupied_cells, random_free_spot};
use crate::basic::{Frames, HexPoint};
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::eat_mechanics::{EatBehavior, EatMechanics};
use crate::snake_control::pathfinder;
use crate::{app, snake, snake_control};
use rand::Rng;

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
        self.0.push(SpawnEvent::Spawn(Apple { pos, apple_type }));
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
            },
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
fn generate_apple_type(prefs: &Prefs, palette: &app::Palette, rng: &mut impl Rng) -> apple::Type {
    if prefs.special_apples {
        choose! {
            let rand: f64 <- rng;
            prefs.prob_spawn_competitor => {
                // apple::Type::SpawnSnake(Box::new(snake::Builder::default()
                //         .snake_type(snake::Type::Competitor { life: Some(200) })
                //         .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                //         .palette(snake::PaletteTemplate::pastel_rainbow(true))
                //         .controller(snake_control::Template::AStar { passthrough_knowledge: PassthroughKnowledge::always(false) })
                //         .speed(1.)
                // ))
                apple::Type::SpawnSnake(Box::new(SnakeBuilder::default()
                    .snake_type(snake::Type::Competitor { life: Some(200) })
                    .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                    .palette(palette.palette_competitor)
                    .controller(snake_control::Template::Algorithm(pathfinder::Template::WeightedBFS))
                    .speed(1.)
                ))
            },
            prefs.prob_spawn_killer => {
                apple::Type::SpawnSnake(Box::new(SnakeBuilder::default()
                    .snake_type(snake::Type::Killer { life: Some(200) })
                    .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                    .palette(palette.palette_killer)
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

pub fn spawn_apples<Rng: rand::Rng>(env: &mut Environment<Rng>) {
    // lazy
    let mut occupied_cells = None;

    let mut spawn = vec![];

    loop {
        let can_spawn = match &env.gtx.apple_spawn_policy {
            SpawnPolicy::None => false,
            SpawnPolicy::Random { apple_count } => env.apples.len() + spawn.len() < *apple_count,
            SpawnPolicy::ScheduledOnEat { apple_count, .. } => {
                env.apples.len() + spawn.len() < *apple_count
            }
        };

        if !can_spawn {
            break;
        }

        let occupied_cells =
            occupied_cells.get_or_insert_with(|| get_occupied_cells(&env.snakes, &env.apples));

        let new_apple = match &mut env.gtx.apple_spawn_policy {
            SpawnPolicy::None => panic!("shouldn't be spawning with SpawnPolicy::None"),
            SpawnPolicy::Random { apple_count } => {
                let apple_pos =
                    match random_free_spot(occupied_cells, env.gtx.board_dim, &mut env.rng) {
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

                let apple_type = generate_apple_type(&env.gtx.prefs, &env.gtx.palette, &mut env.rng);
                Some(Apple { pos: apple_pos, apple_type })
            }
            SpawnPolicy::ScheduledOnEat { schedule: spawns, next_index, current_wait, .. } => {
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
