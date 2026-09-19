//! Functions that are common to all [`Screen`]s for
//! collision detection and snake management

use std::collections::HashSet;
use std::time::Duration;

use anyhow::{Context, Result};
use rand::distributions::uniform::SampleRange;

use crate::app::game_mode::{hunger, GameMode};
use crate::app::portal;
use crate::app::screen::Environment;
use crate::basic::board::{get_occupied_cells, occupied_or_near_players, random_free_spot};
use crate::basic::{Dir, HexPoint};
use crate::rendering::segments::centerline::Centerline;
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::eat_mechanics::{EatBehavior, EatMechanics};
use crate::snake::{self, LengthChange, SegmentType, Snake, State};
use crate::view::snakes::OtherSnakes;
use crate::{rendering, snake_control};

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
    Portal(portal::Behavior),
}

/// Apples are eaten by whichever cell the head is in, and snakes collide by
/// whatever the draw style says. The two are decided independently, so a head
/// can reach an apple and hit something in the same tick.
pub fn find_collisions<Rng>(env: &Environment<Rng>) -> Vec<Collision> {
    let mut collisions = vec![];

    for (snake_index, snake) in living_snakes(env) {
        if let Some(apple_index) = env.apples.iter().position(|apple| apple.pos == snake.head().pos) {
            collisions.push(Collision::Apple { snake_index, apple_index });
        }
    }

    match env.gtx.prefs.draw_style {
        // hexagon segments fill their whole cell, so there cells *are* the shape
        rendering::Style::Hexagon => cell_snake_collisions(env, &mut collisions),
        rendering::Style::Smooth => drawn_snake_collisions(env, &mut collisions),
    }

    collisions
}

fn living_snakes<'a, Rng>(env: &'a Environment<Rng>) -> impl Iterator<Item = (usize, &'a Snake)> + 'a {
    env.snakes
        .iter()
        .enumerate()
        .filter(|(_, s)| !matches!(s.state, State::Crashed | State::Dying | State::Starved))
}

fn collision(snake1_index: usize, snake2_index: usize, snake2_segment_index: usize) -> Collision {
    if snake1_index == snake2_index {
        Collision::Itself {
            snake_index: snake1_index,
            snake_segment_index: snake2_segment_index,
        }
    } else {
        Collision::Snake {
            snake1_index,
            snake2_index,
            snake2_segment_index,
        }
    }
}

/// A head collides with whatever shares its cell.
fn cell_snake_collisions<Rng>(env: &Environment<Rng>, collisions: &mut Vec<Collision>) {
    for (snake1_index, snake1) in living_snakes(env) {
        // several segments can share a cell (one snake passing over another):
        // the worst of them is what happens
        let worst = segments_at(env, snake1_index, snake1.head().pos).max_by_key(|&(_, _, outcome)| outcome);
        if let Some((snake2_index, segment_idx, _)) = worst {
            collisions.push(collision(snake1_index, snake2_index, segment_idx));
        }
    }
}

/// The head's cell still decides *what* it runs into and what that does — a
/// head only ever interacts with what shares its cell, exactly as before. All
/// geometry adds is a veto: candidates whose drawn flesh the head does not
/// actually reach are dropped, so a head no longer crashes into a tail that has
/// already receded out of the way.
///
/// A snake is the `side/2` neighborhood of its centerline (see [`Centerline`]),
/// so the head's cap touches a segment when it comes within the sum of the two
/// half-widths of that segment's centerline.
fn drawn_snake_collisions<Rng>(env: &Environment<Rng>, collisions: &mut Vec<Collision>) {
    let centerlines: Vec<Centerline> = env
        .snakes
        .iter()
        .map(|snake| Centerline::of(&snake.body, &env.gtx))
        .collect();

    for (snake1_index, snake1) in living_snakes(env) {
        let head = &centerlines[snake1_index];
        let Some(probe) = head.head_base() else { continue };

        let worst = segments_at(env, snake1_index, snake1.head().pos)
            .filter(|&(snake2_index, segment_index, _)| {
                let other = &centerlines[snake2_index];
                other
                    .distance_to(segment_index, probe)
                    .is_some_and(|distance| distance <= head.cap_radius + other.half_width)
            })
            .max_by_key(|&(_, _, outcome)| outcome);

        if let Some((snake2_index, segment_index, _)) = worst {
            collisions.push(collision(snake1_index, snake2_index, segment_index));
        }
    }
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
                let outcome = outcome_for(snake, other, itself, segment_index, segment.segment_type);
                (other_index, segment_index, outcome)
            })
    })
}

/// What happens to `snake` when its head runs into segment `segment_index` of
/// `other`.
fn outcome_for(snake: &Snake, other: &Snake, itself: bool, segment_index: usize, segment_type: SegmentType) -> Outcome {
    use EatBehavior::*;

    let behavior = if itself {
        snake.eat_mechanics.eat_self(segment_type)
    } else {
        snake.eat_mechanics.eat_other(other.snake_type, segment_type)
    };
    match behavior {
        Crash | Die => Outcome::Crash,
        // cutting another snake at its head kills both
        Cut if !itself && segment_index == 0 => Outcome::Crash,
        Cut => Outcome::Cut,
        PassUnder | PassOver => Outcome::Pass,
    }
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
            Grow(amount) => env.snakes[snake_index]
                .body
                .length_changes
                .push(LengthChange::new(*amount, hunger::GROW_DURATION)),
            Shrink(amount) => env.snakes[snake_index]
                .body
                .length_changes
                .push(LengthChange::new(-*amount, hunger::SHRINK_DURATION)),
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
        // cells that are actually taken (snake bodies + apples)
        let occupied_cells = get_occupied_cells(&env.snakes, &env.apples);

        // additionally avoid spawning too close to player snake heads, but only
        // as a preference: on a board small enough that the neighborhood wraps
        // around and covers everything, fall back to plain occupancy so we can
        // still spawn wherever there is real free space
        let preferred_free = occupied_or_near_players(&env.snakes, &env.apples, board_dim);

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
        if snake.advance(elapsed, env.gtx.board_dim) {
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

        // hunger mode: a snake shrunk down to the minimum has starved, which ends
        // the game for the player; other snakes just die
        if env.gtx.mode == GameMode::Hunger && snake.state == State::Living && snake.body.length <= hunger::MIN_LENGTH {
            if snake.snake_type == snake::Type::Player {
                snake.starve();
            } else {
                snake.die();
            }
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

/// Only heads eat apples: an apple that a tail has grown over (hunger mode)
/// moves somewhere else.
pub fn relocate_covered_apples<Rng: rand::Rng>(env: &mut Environment<Rng>) {
    let covered: HashSet<HexPoint> = env
        .snakes
        .iter()
        .flat_map(|snake| snake.body.segments.iter().skip(1))
        .map(|segment| segment.pos)
        .collect();
    if !env.apples.iter().any(|apple| covered.contains(&apple.pos)) {
        return;
    }

    let mut occupied = get_occupied_cells(&env.snakes, &env.apples);
    let mut no_room = vec![];
    for (apple_index, apple) in env.apples.iter_mut().enumerate() {
        if !covered.contains(&apple.pos) {
            continue;
        }
        match random_free_spot(&occupied, env.gtx.board_dim, &mut env.rng) {
            Some(pos) => {
                apple.pos = pos;
                if let Err(idx) = occupied.binary_search(&pos) {
                    occupied.insert(idx, pos);
                }
            }
            None => no_room.push(apple_index),
        }
    }
    env.remove_apples(no_room);
}
