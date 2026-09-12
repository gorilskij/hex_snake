//! Functions that are common to all [`Screen`]s for
//! collision detection and snake management

use std::time::Duration;

use anyhow::{Context, Result};
use rand::distributions::uniform::SampleRange;

use crate::app::portal;
use crate::app::screen::Environment;
use crate::basic::board::{get_occupied_cells, random_free_spot};
use crate::basic::{Dir, HexPoint};
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

    for (snake1_index, snake1) in env
        .snakes
        .iter()
        .enumerate()
        .filter(|(_, s)| !matches!(s.state, State::Crashed | State::Dying))
    {
        let pos = snake1.head().pos;

        // snakes and apples cannot overlap
        if let Some(apple_index) = env.apples.iter().position(|apple| apple.pos == pos) {
            collisions.push(Collision::Apple {
                snake_index: snake1_index,
                apple_index,
            });
            continue;
        }

        // several segments can share a cell (one snake passing over another):
        // the worst of them is what happens
        let worst = segments_at(env, snake1_index, pos).max_by_key(|&(_, _, outcome)| outcome);
        if let Some((snake2_index, segment_idx, _)) = worst {
            collisions.push(if snake2_index == snake1_index {
                Collision::Itself {
                    snake_index: snake1_index,
                    snake_segment_index: segment_idx,
                }
            } else {
                Collision::Snake {
                    snake1_index,
                    snake2_index,
                    snake2_segment_index: segment_idx,
                }
            });
        }
    }

    collisions
}

/// What would happen to a snake whose head entered a cell, in increasing order
/// of severity.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Outcome {
    Apple,
    Pass,
    Cut,
    Crash,
}

/// What would happen to `snake_index` if its head entered `pos` right now, or
/// `None` if nothing is there. Decided exactly as [`find_collisions`] decides
/// an actual collision.
pub fn outcome_at<Rng>(env: &Environment<Rng>, snake_index: usize, pos: HexPoint) -> Option<Outcome> {
    if env.apples.iter().any(|apple| apple.pos == pos) {
        return Some(Outcome::Apple);
    }
    segments_at(env, snake_index, pos).map(|(_, _, outcome)| outcome).max()
}

/// Every segment at `pos` that `snake_index`'s head would run into there, as
/// `(snake index, segment index, outcome)`.
fn segments_at<Rng>(
    env: &Environment<Rng>,
    snake_index: usize,
    pos: HexPoint,
) -> impl Iterator<Item = (usize, usize, Outcome)> + '_ {
    use EatBehavior::*;

    let snake = &env.snakes[snake_index];
    env.snakes.iter().enumerate().flat_map(move |(other_index, other)| {
        let itself = other_index == snake_index;
        other
            .body
            .segments
            .iter()
            .enumerate()
            // a snake's head never collides with itself
            .skip(itself as usize)
            .filter(move |(_, segment)| segment.pos == pos)
            .map(move |(segment_index, segment)| {
                let behavior = if itself {
                    snake.eat_mechanics.eat_self(segment.segment_type)
                } else {
                    snake.eat_mechanics.eat_other(other.snake_type, segment.segment_type)
                };
                let outcome = match behavior {
                    Crash | Die => Outcome::Crash,
                    // cutting another snake at its head kills both
                    Cut if !itself && segment_index == 0 => Outcome::Crash,
                    Cut => Outcome::Cut,
                    PassUnder | PassOver => Outcome::Pass,
                };
                (other_index, segment_index, outcome)
            })
    })
}

/// Apply the effect of every apple collision and remove the eaten apples.
/// Returns the snakes to spawn (from spawn-snake / rain apples).
///
/// Each snake head collides with at most one thing per [`find_collisions`],
/// so apple and snake collisions concern disjoint snakes — this pass and
/// [`handle_snake_collisions`] are independent and may run in either order.
#[must_use]
pub fn handle_apple_collisions<Rng: rand::Rng>(
    env: &mut Environment<Rng>,
    collisions: &[Collision],
) -> Vec<SnakeBuilder> {
    let board_width = env.gtx.board_dim.h;

    let mut spawn_snakes = vec![];
    let mut to_remove = vec![];
    for collision in collisions.iter().copied() {
        let Collision::Apple { snake_index, apple_index } = collision else {
            continue;
        };

        to_remove.push(apple_index);

        use crate::apple::Type::*;
        match &env.apples[apple_index].apple_type {
            Eat(food) => {
                env.snakes[snake_index].body.segments[0].segment_type = SegmentType::Eaten {
                    original_food: *food,
                    food_left: *food,
                }
            }
            Shrink(amount) => {
                /////////////////////////////////////////////////////////////////////
            }
            SpawnSnake(seed) => spawn_snakes.push((**seed).clone()),
            SpawnRain => {
                let seed = SnakeBuilder::default()
                    .snake_type(snake::Type::Rain)
                    .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                    .palette(env.gtx.palette.palette_rain)
                    .controller(snake_control::Template::Rain)
                    .dir(Dir::D);

                for h in (0..board_width).step_by(5) {
                    spawn_snakes.push(
                        seed.clone()
                            .pos(HexPoint { h, v: 0 })
                            .len((3..10).sample_single(&mut env.rng))
                            .speed((0.2..1.5).sample_single(&mut env.rng)),
                    );
                }
            }
        }
    }

    env.remove_apples(to_remove);

    spawn_snakes
}

/// Resolve every snake-snake and self collision via the snakes' eat mechanics.
/// Returns whether a snake crashed and ended the game.
#[must_use]
pub fn handle_snake_collisions<Rng>(env: &mut Environment<Rng>, collisions: &[Collision]) -> bool {
    use EatBehavior::*;

    let snakes = &mut env.snakes;
    let mut game_over = false;
    for collision in collisions.iter().copied() {
        match collision {
            Collision::Apple { .. } => {}
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
                let segment_type = snake.body.segments[snake_segment_index].segment_type;
                let behavior = snake.eat_mechanics.eat_self(segment_type);
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
            Collision::Portal(_behavior) => todo!(),
        }
    }

    game_over
}

pub fn spawn_snakes(env: &mut Environment, snake_builders: Vec<SnakeBuilder>) -> Result<()> {
    let board_dim = env.gtx.board_dim;

    for mut snake_builder in snake_builders {
        // avoid spawning too close to player snake heads
        const PLAYER_SNAKE_HEAD_NO_SPAWN_RADIUS: usize = 7;

        // cells that are actually taken (snake bodies + apples)
        let occupied_cells = get_occupied_cells(&env.snakes, &env.apples);

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
    for (snake_idx, snake) in snakes.iter_mut().enumerate() {
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

            snake.advance_cell(&env.portals, &env.gtx);
        }

        // remove the snake once the death hole has swallowed all of it (a snake
        // that hasn't emerged yet is also `on_board == 0`, hence the state check)
        if snake.state == State::Dying && snake.body.on_board() <= 0.0 {
            remove_snakes.push(snake_idx);
        }
    }

    remove_snakes.sort_unstable();
    remove_snakes.into_iter().rev().for_each(|i| {
        env.remove_snake(i);
    });

    new_cell_occupied
}

/// Poll the controller of every snake that hasn't yet committed a direction
/// for its current cell.
///
/// This must run **after** the collision handlers, not as part of
/// [`advance_snakes`]: a controller's decision is locked in for the rest of
/// the cell, so it has to see the post-collision world. Deciding between the
/// movement and the collision pass meant the apple seeker was consulted while
/// standing on an apple that had not been removed yet — it kept targeting it,
/// maintained course, and overshot before recalculating a cell too late.
pub fn update_snake_dirs(env: &mut Environment) {
    let snakes = &mut env.snakes;
    for snake_idx in 0..snakes.len() {
        let (snake, other_snakes) = OtherSnakes::split_snakes(snakes, snake_idx);
        if !snake.dir_updated {
            snake.update_dir(other_snakes, &env.apples, &env.gtx);
        }
    }
}
