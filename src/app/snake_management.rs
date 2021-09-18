//! Functions that are common to all [`Screen`]s for
//! collision detection and snake management

use crate::{
    app::{
        apple::{self, Apple},
        screen::Environment,
        snake::{self, utils::split_snakes_mut, EatBehavior, Seed, SegmentType, Snake, State},
        utils::{get_occupied_cells, random_free_spot},
    },
    basic::{Dir, FrameStamp, HexDim},
};
use rand::Rng;

#[derive(Copy, Clone)]
pub enum Collision {
    Apple {
        snake_index: usize,
        apple_index: usize,
    },
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
}

pub fn find_collisions<E: Environment>(env: &E) -> Vec<Collision> {
    let snakes = env.snakes();
    let apples = env.apples();

    let mut collisions = vec![];

    // check whether snake1 collided with an apple or with snake2
    'outer: for (snake1_index, snake1) in snakes
        .iter()
        .enumerate()
        .filter(|(_, s)| !matches!(s.state, State::Crashed | State::Dying))
    {
        for (apple_index, apple) in apples.iter().enumerate() {
            if snake1.head().pos == apple.pos {
                collisions.push(Collision::Apple {
                    snake_index: snake1_index,
                    apple_index,
                })
            }
        }

        for (snake2_index, other) in snakes.iter().enumerate() {
            let mut iter = other.body.cells.iter().enumerate();

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

/// Returns `(spawn_snakes, game_over)` where
///  - `spawn_snakes` describes the new snakes to spawn
/// (competitors, killers, etc.)
///  - `game_over` tells whether a snake crashed and ended the game
#[must_use]
pub fn handle_collisions<E: Environment>(
    env: &mut E,
    collisions: &[Collision],
) -> (Vec<snake::Seed>, bool) {
    let (snakes, apples) = env.snakes_apples_mut();

    let mut spawn_snakes = vec![];
    let mut remove_apples = vec![];
    let mut game_over = false;
    for collision in collisions.iter().copied() {
        match collision {
            Collision::Apple { snake_index, apple_index } => {
                remove_apples.push(apple_index);
                match &apples[apple_index].apple_type {
                    apple::Type::Normal(food) => {
                        snakes[snake_index].body.cells[0].segment_type = SegmentType::Eaten {
                            original_food: *food,
                            food_left: *food,
                        }
                    }
                    apple::Type::SpawnSnake(seed) => spawn_snakes.push(seed.clone()),
                }
            }
            Collision::Snake {
                snake1_index,
                snake2_index,
                snake2_segment_index,
            } => {
                let mechanics = &snakes[snake1_index].eat_mechanics;
                let behavior = mechanics
                    .eat_other
                    .get(&snakes[snake2_index].snake_type)
                    .copied()
                    .unwrap_or(mechanics.default);
                match behavior {
                    EatBehavior::Cut => snakes[snake2_index].cut_at(snake2_segment_index),
                    EatBehavior::Crash => {
                        snakes[snake1_index].crash();
                        game_over = true;
                    }
                    EatBehavior::Die => snakes[snake1_index].die(),
                }
            }
            Collision::Itself { snake_index, snake_segment_index } => {
                let behavior = snakes[snake_index].eat_mechanics.eat_self;
                match behavior {
                    EatBehavior::Cut => snakes[snake_index].cut_at(snake_segment_index),
                    EatBehavior::Crash => {
                        snakes[snake_index].crash();
                        game_over = true;
                    }
                    EatBehavior::Die => snakes[snake_index].die(),
                }
            }
        }
    }

    remove_apples.sort_unstable();
    for apple_idx in remove_apples.into_iter().rev() {
        env.remove_apple(apple_idx);
    }

    (spawn_snakes, game_over)
}

pub fn spawn_snakes<E: Environment>(env: &mut E, seeds: Vec<Seed>) {
    let board_dim = env.board_dim();

    for mut seed in seeds {
        // avoid spawning too close to player snake heads
        const PLAYER_SNAKE_HEAD_NO_SPAWN_RADIUS: usize = 7;

        let mut occupied_cells = get_occupied_cells(env.snakes(), env.apples());
        for snake in env
            .snakes()
            .iter()
            .filter(|s| s.snake_type == snake::Type::Player)
        {
            let neighborhood = snake.reachable(PLAYER_SNAKE_HEAD_NO_SPAWN_RADIUS, board_dim);
            occupied_cells.extend_from_slice(&neighborhood);
        }
        occupied_cells.sort_unstable();
        occupied_cells.dedup();

        let rng = env.rng();
        if let Some(pos) = random_free_spot(&occupied_cells, board_dim, rng) {
            seed.pos = Some(pos);
            seed.dir = Some(Dir::random(rng));
            seed.len = Some(rng.gen_range(7, 15));
            env.add_snake(&seed);
        } else {
            eprintln!("warning: failed to spawn snake, no free spaces left")
        }
    }
}

/// Returns the indices of snakes to be deleted (in reverse order so they
/// can be deleted straight away)
pub fn advance_snakes<E: Environment>(env: &mut E) {
    let board_dim = env.board_dim();
    let frame_stamp = env.frame_stamp();
    let (snakes, apples) = env.snakes_apples_mut();

    let mut remove_snakes = vec![];
    for snake_idx in 0..snakes.len() {
        // set snake to die if it ran out of life
        match &mut snakes[snake_idx].snake_type {
            snake::Type::Competitor { life: Some(life) }
            | snake::Type::Killer { life: Some(life) } => {
                if *life == 0 {
                    snakes[snake_idx].die();
                } else {
                    *life -= 1;
                }
            }
            _ => (),
        }

        let (snake, other_snakes) = split_snakes_mut(snakes, snake_idx);

        // advance the snake
        snake.advance(other_snakes, apples, board_dim, frame_stamp);

        // remove snake if it ran out of body
        if snake.len() == 0 {
            remove_snakes.push(snake_idx);
        }
    }

    remove_snakes.sort_unstable();
    remove_snakes.into_iter().rev().for_each(|i| {
        env.remove_snake(i);
    });
}
