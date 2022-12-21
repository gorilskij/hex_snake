//! Functions that are common to all [`Screen`]s for
//! collision detection and snake management

use crate::app::screen::Environment;
use crate::basic::board::{get_occupied_cells, random_free_spot};
use crate::basic::{Dir, HexPoint};
use crate::error::{Error, ErrorConversion, Result};
use crate::snake::{self, EatBehavior, EatMechanics, SegmentType, State};
use crate::snake_control;
use crate::view::snakes::OtherSnakes;
use ggez::Context;
use rand::distributions::uniform::SampleRange;

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

// TODO: maybe replace Environment with GameContext
/// Returns `(spawn_snakes, game_over)` where
///  - `spawn_snakes` describes the new snakes to spawn
/// (competitors, killers, etc.)
///  - `game_over` tells whether a snake crashed and ended the game
#[must_use]
pub fn handle_collisions<E: Environment>(
    env: &mut E,
    collisions: &[Collision],
) -> (Vec<snake::Builder>, bool) {
    let board_width = env.board_dim().h;
    let (snakes, apples, rng) = env.snakes_apples_rng_mut();

    let mut spawn_snakes = vec![];
    let mut remove_apples = vec![];
    let mut game_over = false;
    for collision in collisions.iter().copied() {
        use EatBehavior::*;
        match collision {
            Collision::Apple { snake_index, apple_index } => {
                remove_apples.push(apple_index);

                use crate::apple::Type::*;
                match &apples[apple_index].apple_type {
                    Food(food) => {
                        snakes[snake_index].body.segments[0].segment_type = SegmentType::Eaten {
                            original_food: *food,
                            food_left: *food,
                        }
                    }
                    SpawnSnake(seed) => spawn_snakes.push((**seed).clone()),
                    SpawnRain => {
                        let seed = snake::Builder::default()
                            .snake_type(snake::Type::Rain)
                            .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                            // TODO: factor out palette into game palette
                            // .palette(snake::PaletteTemplate::alternating_white())
                            .palette(snake::PaletteTemplate::gray_gradient(0.5, false))
                            .controller(snake_control::Template::Rain)
                            .dir(Dir::D);

                        for h in (0..board_width).step_by(5) {
                            spawn_snakes.push(
                                seed.clone()
                                    .pos(HexPoint { h, v: 0 })
                                    .len((3..10).sample_single(rng))
                                    .speed((0.2..1.5).sample_single(rng)),
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
                let snake2_segment_type = snake2.body.segments[snake2_segment_index]
                    .segment_type
                    .raw_type();
                let behavior = snake1.eat_mechanics.eat_other[&snake2_type][&snake2_segment_type];

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
                        game_over = true;
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
                let segment_type = snake.body.segments[snake_segment_index]
                    .segment_type
                    .raw_type();
                let behavior = snake.eat_mechanics.eat_self[&segment_type];
                match behavior {
                    Cut => snakes[snake_index].cut_at(snake_segment_index),
                    Crash => {
                        snakes[snake_index].crash();
                        game_over = true;
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
        }
    }

    remove_apples.sort_unstable();
    for apple_idx in remove_apples.into_iter().rev() {
        env.remove_apple(apple_idx);
    }

    (spawn_snakes, game_over)
}

pub fn spawn_snakes<E: Environment>(env: &mut E, snake_builders: Vec<snake::Builder>) -> Result {
    let board_dim = env.board_dim();

    for mut snake_builder in snake_builders {
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

        match snake_builder.pos {
            Some(pos) => {
                let is_occupied = env
                    .snakes()
                    .iter()
                    .flat_map(|snake| snake.body.segments.iter().map(|seg| seg.pos))
                    .any(|p| p == pos);

                if is_occupied {
                    eprintln!("warning: failed to spawn snake, no free spaces left");
                    continue;
                }
            }
            None => {
                if let Some(pos) = random_free_spot(&occupied_cells, board_dim, env.rng()) {
                    snake_builder.pos = Some(pos);
                } else {
                    eprintln!("warning: failed to spawn snake, no free spaces left");
                    continue;
                }
            }
        }

        snake_builder
            .dir
            .get_or_insert_with(|| Dir::random(env.rng()));
        snake_builder
            .len
            .get_or_insert_with(|| (7..15).sample_single(env.rng()));

        env.add_snake(&snake_builder)
            .map_err(Error::from)
            .with_trace_step("spawn_snakes")?;
    }

    Ok(())
}

/// Returns the indices of snakes to be deleted (in reverse order so they
/// can be deleted straight away)
pub fn advance_snakes<E: Environment>(env: &mut E, ctx: &Context) {
    let (snakes, apples, gtx) = env.snakes_apples_gtx_mut();

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

        let (snake, other_snakes) = OtherSnakes::split_snakes(snakes, snake_idx);

        // advance the snake
        snake.advance(other_snakes, apples, gtx, ctx);

        // remove snake if it ran out of body
        if snake.body.visible_len() == 0 {
            remove_snakes.push(snake_idx);
        }
    }

    remove_snakes.sort_unstable();
    remove_snakes.into_iter().rev().for_each(|i| {
        env.remove_snake(i);
    });
}
