//! Functions that are common to all [`Screen`]s for
//! collision detection and snake management

use std::time::Duration;

use anyhow::{Context, Result};
use rand::distributions::uniform::SampleRange;

use crate::app::portal;
use crate::app::screen::Environment;
use crate::basic::board::{get_occupied_cells, random_free_spot};
use crate::basic::{Dir, Food, HexPoint};
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::eat_mechanics::{EatBehavior, EatMechanics};
use crate::snake::{self, SegmentType, State};
use crate::snake_control;
use crate::view::snakes::OtherSnakes;

#[derive(Copy, Clone)]
pub enum Collision {
    Apple {
        snake_index: usize,
        apple_index: usize,
    },
    // TODO: implement separate head-head collision mechanism
    // head of snake1 collided with head or body of snake2
    Snake {
        snake1_index: usize,
        snake2_index: usize,
        snake2_segment_index: usize,
    },
    // snake collided with itself
    Itself {
        snake_index: usize,
        snake_segment_index: usize,
    },
    Portal(portal::Behavior),
}

pub fn find_collisions<Rng>(env: &Environment<Rng>) -> Vec<Collision> {
    let mut collisions = vec![];

    // check whether snake1 collided with an apple or with snake2
    'outer: for (snake1_index, snake1) in env
        .snakes
        .iter()
        .enumerate()
        .filter(|(_, s)| !matches!(s.state, State::Crashed | State::Dying))
    {
        for (apple_index, apple) in env.apples.iter().enumerate() {
            if snake1.head().pos == apple.pos {
                collisions.push(Collision::Apple {
                    snake_index: snake1_index,
                    apple_index,
                });
                // snakes and apples cannot overlap
                continue 'outer;
            }
        }

        for (snake2_index, other) in env.snakes.iter().enumerate() {
            let mut iter = other.body.segments.iter().enumerate();

            // ignore head-head collision with itself
            if snake1_index == snake2_index {
                let _ = iter.next();
            }

            for (segment_idx, segment) in iter {
                if snake1.head().pos == segment.pos {
                    if snake1_index == snake2_index {
                        collisions.push(Collision::Itself {
                            snake_index: snake1_index,
                            snake_segment_index: segment_idx,
                        })
                    } else {
                        collisions.push(Collision::Snake {
                            snake1_index,
                            snake2_index,
                            snake2_segment_index: segment_idx,
                        });
                    }

                    continue 'outer;
                }
            }
        }
    }

    collisions
}

/// What happened as a result of this update's collisions, for the game layer
/// to act on (spawning snakes, scoring, notifications)
#[must_use]
#[derive(Default)]
pub struct CollisionOutcome {
    /// New snakes to spawn (competitors, killers, etc.)
    pub spawn_snakes: Vec<SnakeBuilder>,
    /// Whether a snake crashed and ended the game
    pub game_over: bool,
    /// Food value of each apple eaten by a player snake (fed to the combo
    /// system, one entry per apple)
    pub player_eaten: Vec<Food>,
    /// Whether a player snake picked up a speed boost
    pub player_boosted: bool,
    /// How many frenzy apples were eaten (each bursts extra food apples)
    pub frenzies: usize,
}

// TODO: maybe replace Environment with GameContext
pub fn handle_collisions<Rng: rand::Rng>(env: &mut Environment<Rng>, collisions: &[Collision]) -> CollisionOutcome {
    let board_width = env.gtx.board_dim.h;

    let mut outcome = CollisionOutcome::default();
    let mut to_remove = vec![];
    for collision in collisions.iter().copied() {
        use EatBehavior::*;
        let snakes = &mut env.snakes;

        match collision {
            Collision::Apple { snake_index, apple_index } => {
                to_remove.push(apple_index);

                use crate::apple::Type::*;
                match &env.apples[apple_index].apple_type {
                    Food(food) => {
                        snakes[snake_index].body.segments[0].segment_type = SegmentType::Eaten {
                            original_food: *food,
                            food_left: *food,
                        };
                        if snakes[snake_index].snake_type == snake::Type::Player {
                            outcome.player_eaten.push(*food);
                        }
                    }
                    SpeedBoost => {
                        /// How much faster and for how long a speed-boost apple makes a snake
                        const BOOST_FACTOR: f32 = 1.6;
                        const BOOST_DURATION: Duration = Duration::from_secs(8);

                        snakes[snake_index].boost_speed(BOOST_FACTOR, BOOST_DURATION);
                        if snakes[snake_index].snake_type == snake::Type::Player {
                            outcome.player_boosted = true;
                        }
                    }
                    Frenzy => outcome.frenzies += 1,
                    SpawnSnake(seed) => outcome.spawn_snakes.push((**seed).clone()),
                    SpawnRain => {
                        let seed = SnakeBuilder::default()
                            .snake_type(snake::Type::Rain)
                            .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                            .palette(env.gtx.palette.palette_rain)
                            .controller(snake_control::Template::Rain)
                            .dir(Dir::D);

                        for h in (0..board_width).step_by(5) {
                            outcome.spawn_snakes.push(
                                seed.clone()
                                    .pos(HexPoint { h, v: 0 })
                                    .len((3..10).sample_single(&mut env.rng))
                                    .speed((0.2..1.5).sample_single(&mut env.rng)),
                            );
                        }
                    }
                }
            }
            Collision::Snake {
                snake1_index,
                snake2_index,
                snake2_segment_index,
            } => {
                let snake1 = &snakes[snake1_index];
                let snake2 = &snakes[snake2_index];
                let snake2_type = snake2.snake_type;
                let snake2_segment_type = snake2.body.segments[snake2_segment_index].segment_type;
                let behavior = snake1.eat_mechanics.eat_other(snake2_type, snake2_segment_type);

                match behavior {
                    Cut => {
                        // if it's a head-head collision, both snakes die
                        if snake2_segment_index == 0 {
                            snakes[snake1_index].die();
                            snakes[snake2_index].die();
                        } else {
                            snakes[snake2_index].cut_at(snake2_segment_index)
                        }
                    }
                    Crash => {
                        snakes[snake1_index].crash();
                        outcome.game_over = true;
                    }
                    Die => snakes[snake1_index].die(),
                    PassUnder => {
                        snakes[snake1_index].body.segments[0].z_index =
                            snakes[snake2_index].body.segments[snake2_segment_index].z_index - 1
                    }
                    PassOver => {
                        snakes[snake1_index].body.segments[0].z_index =
                            snakes[snake2_index].body.segments[snake2_segment_index].z_index + 1
                    }
                }
            }
            Collision::Itself { snake_index, snake_segment_index } => {
                let snake = &snakes[snake_index];
                let segment_type = snake.body.segments[snake_segment_index].segment_type;
                let behavior = snake.eat_mechanics.eat_self(segment_type);
                match behavior {
                    Cut => snakes[snake_index].cut_at(snake_segment_index),
                    Crash => {
                        snakes[snake_index].crash();
                        outcome.game_over = true;
                    }
                    Die => snakes[snake_index].die(),
                    PassUnder => {
                        snakes[snake_index].body.segments[0].z_index =
                            snakes[snake_index].body.segments[snake_segment_index].z_index - 1
                    }
                    PassOver => {
                        snakes[snake_index].body.segments[0].z_index =
                            snakes[snake_index].body.segments[snake_segment_index].z_index + 1
                    }
                }
            }
            Collision::Portal(_behavior) => todo!(),
        }
    }

    env.remove_apples(to_remove);

    outcome
}

pub fn spawn_snakes(env: &mut Environment, snake_builders: Vec<SnakeBuilder>) -> Result<()> {
    let board_dim = env.gtx.board_dim;

    for mut snake_builder in snake_builders {
        // avoid spawning too close to player snake heads
        const PLAYER_SNAKE_HEAD_NO_SPAWN_RADIUS: usize = 7;

        // cells that are actually taken (snake bodies + apples)
        let occupied_cells = get_occupied_cells(&env.snakes, &env.apples, &env.alt_portals);

        // additionally avoid spawning too close to player snake heads, but only
        // as a preference: on a board small enough that the neighborhood wraps
        // around and covers everything, fall back to plain occupancy so we can
        // still spawn wherever there is real free space
        let mut preferred_free = occupied_cells.clone();
        for snake in env.snakes.iter().filter(|s| s.snake_type == snake::Type::Player) {
            let neighborhood = snake.reachable(PLAYER_SNAKE_HEAD_NO_SPAWN_RADIUS, board_dim);
            preferred_free.extend_from_slice(&neighborhood);
        }
        preferred_free.sort_unstable();
        preferred_free.dedup();

        match snake_builder.pos {
            Some(pos) => {
                let is_occupied = env
                    .snakes
                    .iter()
                    .flat_map(|snake| snake.body.segments.iter().map(|seg| seg.pos))
                    .any(|p| p == pos);

                if is_occupied {
                    eprintln!("warning: failed to spawn snake, requested cell is occupied");
                    continue;
                }
            }
            None => {
                let spot = random_free_spot(&preferred_free, board_dim, &mut env.rng)
                    .or_else(|| random_free_spot(&occupied_cells, board_dim, &mut env.rng));
                if let Some(pos) = spot {
                    snake_builder.pos = Some(pos);
                } else {
                    eprintln!("warning: failed to spawn snake, no free spaces left");
                    continue;
                }
            }
        }

        snake_builder.dir.get_or_insert_with(|| Dir::random(&mut env.rng));
        snake_builder
            .len
            .get_or_insert_with(|| (7..15).sample_single(&mut env.rng));

        env.add_snake(&snake_builder).context("spawn_snakes")?;
    }

    Ok(())
}

/// Return value indicates whether any snake has crossed into a new cell
pub fn advance_snakes(env: &mut Environment, elapsed: Duration) -> bool {
    let snakes = &mut env.snakes;

    let mut new_cell_occupied = false;

    let mut remove_snakes = vec![];
    for snake_idx in 0..snakes.len() {
        let (snake, other_snakes) = OtherSnakes::split_snakes(snakes, snake_idx);

        // advance the snake
        if snake.advance(elapsed) {
            // block is entered if the snake crossed a cell boundary
            new_cell_occupied = true;

            match &mut snake.snake_type {
                snake::Type::Competitor { life: Some(life) } | snake::Type::Killer { life: Some(life) } => {
                    if *life == 0 {
                        // set snake to die if it ran out of life
                        snake.die();
                    } else {
                        *life -= 1;
                    }
                }
                _ => (),
            }

            snake.advance_cell(&env.portals, &env.alt_portals, &env.gtx);
        }

        if !snake.dir_updated {
            snake.update_dir(other_snakes, &env.apples, &env.gtx);
        }

        // remove snake if it ran out of body
        if snake.body.visible_len() == 0 {
            remove_snakes.push(snake_idx);
        }
    }

    remove_snakes.sort_unstable();
    remove_snakes.into_iter().rev().for_each(|i| {
        env.remove_snake(i);
    });

    new_cell_occupied
}
