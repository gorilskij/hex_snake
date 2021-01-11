use crate::app::hex::{Dir, HexPos};
use crate::app::snake::{SnakeController, Snake, SnakeRepr};
use rand::{thread_rng, Rng};

pub struct SnakeAI;

// very basic ai, goes to apples, avoids snakes
// works pretty well
impl  SnakeAI {
    pub fn new() -> Self {
        Self
    }
}

fn dir_score(head: HexPos, dir: Dir, board_dim: HexPos, snakes: &Vec<&SnakeRepr>, apples: &[HexPos]) -> usize {
    let mut distance = 0;
    let mut new_head = head;
    while !apples.contains(&new_head) {
        distance += 1;
        new_head.step_and_teleport(dir, board_dim);

        for snake in snakes {
            if snake.body.contains(&new_head) {
                return distance // the higher the distance to a body part, the higher the score
            }
        }
    }
    // println!("for dir {:?}, dist: {}", dir, distance);
    // the lower the distance to an apple, the higher the score
    board_dim.h as usize + board_dim.v as usize - distance
}

impl SnakeController for SnakeAI {
    fn next_dir<'a, 'b>(&'a mut self, snake: &'b SnakeRepr, mut other_snakes: Vec<&'b SnakeRepr>, apples: &'b [HexPos], board_dim: HexPos) -> Option<Dir> {
        use Dir::*;
        let available_directions: Vec<_> = [UL, U, UR, DL, D, DR]
            .iter()
            .filter(|&&d| d != -snake.dir)
            .copied()
            .collect();

        // no sharp turns
        // let available_directions = match snake.dir {
        //     UL => [DL, UL, U],
        //     U => [UL, U, UR],
        //     UR => [U, UR, DR],
        //     DR => [UR, DR, D],
        //     D => [DR, D, DL],
        //     DL => [D, DL, UL],
        // };

        other_snakes.push(snake); // avoid all snakes, including self

        let new_dir = available_directions
            .iter()
            .max_by_key(|&&dir| dir_score(snake.body[0], dir, board_dim, &other_snakes, apples))
            .copied();

        // if let Some(dir) = new_dir {
        //     println!("new: {:?}", dir)
        // }
        new_dir
    }
}