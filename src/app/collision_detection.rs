use crate::app::snake::Snake;
use crate::app::screen::game::Apple;
use crate::app::snake::State;

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
    }
}

pub fn find_collisions(snakes: &[Snake], apples: &[Apple]) -> Vec<Collision> {
    let mut collisions = vec![];

    // checks if snake i crashed into snake j
    // crashed and dying snakes can be ignored for i
    'outer: for (snake1_index, snake1) in snakes
        .iter()
        .enumerate()
        .filter(|(_, s)| !matches!(s.state, State::Crashed | State::Dying))
    {
        for (apple_index, apple) in apples.iter().enumerate() {
            if snake1.head().pos == apple.pos {
                collisions.push(Collision::Apple { snake_index: snake1_index, apple_index })
            }
        }

        for (snake2_index, other) in snakes.iter().enumerate() {
            let mut iter = other.body.cells.iter().enumerate();
            // ignore own head-head collision
            if snake1_index == snake2_index { let _ = iter.next(); }
            for (segment_idx, segment) in iter {
                if snake1.head().pos == segment.pos {
                    // avoid duplicate head-head collision
                    if segment_idx == 0 && collisions.iter().any(|collision| matches!(collision, Collision::Snake {snake1_index: snake2_index, snake2_index: snake1_index, snake2_segment_index: 0})) {
                        continue 'outer;
                    }
                    collisions.push(Collision::Snake {snake1_index,snake2_index,snake2_segment_index:segment_idx});
                    continue 'outer;
                }
            }
        }
    }

    collisions
}
