use crate::app::{
    screen::{
        Apple, Type,
    },
    snake::{self, EatBehavior, SegmentType, Snake, State},
};
use crate::app::utils::{get_occupied_cells, random_free_spot};
use crate::basic::{HexDim, Dir};
use crate::app::snake::Seed;
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

pub fn find_collisions(snakes: &[Snake], apples: &[Apple]) -> Vec<Collision> {
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
                    // prevent duplicate head-head collision
                    if segment_idx == 0
                        && collisions.iter().any(|collision| {
                            matches!(
                                collision,
                                Collision::Snake {
                                    snake1_index: _snake2_index,
                                    snake2_index: _snake1_index,
                                    snake2_segment_index: 0
                                }
                            )
                        })
                    {
                        continue 'outer;
                    }

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

/// The third return value indicates whether it's game over (whether a snake has crashed)
pub fn handle_collisions<'a>(
    collisions: &[Collision],
    snakes: &mut [Snake],
    apples: &'a [Apple],
) -> (Vec<&'a snake::Seed>, Vec<usize>, bool) {
    let mut spawn_snakes = vec![];
    let mut remove_apples = vec![];
    let mut game_over = false;
    for collision in collisions.iter().copied() {
        match collision {
            Collision::Apple { snake_index, apple_index } => {
                remove_apples.push(apple_index);
                match &apples[apple_index].apple_type {
                    Type::Normal(food) => {
                        snakes[snake_index].body.cells[0].typ = SegmentType::Eaten {
                            original_food: *food,
                            food_left: *food,
                        }
                    }
                    Type::SpawnSnake(seed) => spawn_snakes.push(seed),
                }
            }
            Collision::Snake {
                snake1_index,
                snake2_index,
                snake2_segment_index,
            } => {
                let behavior = if snake2_segment_index == 0 {
                    // special case for head-head collision (always crash)
                    EatBehavior::Crash
                } else {
                    let mechanics = &snakes[snake1_index].eat_mechanics;
                    mechanics
                        .eat_other
                        .get(&snakes[snake2_index].snake_type)
                        .copied()
                        .unwrap_or(mechanics.default)
                };
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
    (spawn_snakes, remove_apples, game_over)
}

// this is here because it's a result of collision handling
pub fn spawn_snakes(seeds: Vec<&Seed>, snakes: &mut Vec<Snake>, apples: &[Apple], board_dim: HexDim, rng: &mut impl Rng) {
    for seed in seeds {
        // avoid spawning too close to player snake heads
        const PLAYER_SNAKE_HEAD_NO_SPAWN_RADIUS: usize = 7;

        let mut occupied_cells = get_occupied_cells(snakes, apples);
        for snake in snakes
            .iter()
            .filter(|s| s.snake_type == snake::Type::Player)
        {
            let neighborhood = snake.reachable(PLAYER_SNAKE_HEAD_NO_SPAWN_RADIUS, board_dim);
            occupied_cells.extend_from_slice(&neighborhood);
        }
        occupied_cells.sort_unstable();
        occupied_cells.dedup();

        if let Some(pos) = random_free_spot(&occupied_cells, board_dim, rng) {
            snakes
                .push(Snake::from_seed(&seed, pos, Dir::random(rng), 10))
        } else {
            eprintln!("warning: failed to spawn snake, no free spaces left")
        }
    }
}
