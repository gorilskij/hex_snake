use crate::{
    app::{
        game::Apple,
        snake::{
            controller::{Controller, OtherSnakes},
            SnakeBody,
        },
    },
    basic::{Dir, HexDim, HexPoint},
};
use std::iter::once;

pub struct Competitor1;

// TODO: this could be made faster by checking for each apple and snake segment
//  whether it is in a straight line from the head and calculating the
//  distance only for those
fn dir_score(
    head: HexPoint,
    dir: Dir,
    board_dim: HexDim,
    snake_positions: &[HexPoint],
    apple_positions: &[HexPoint],
) -> usize {
    let mut distance = 0;
    let mut new_head = head;

    while !apple_positions.contains(&new_head) {
        distance += 1;
        new_head = new_head.wrapping_translate(dir, 1, board_dim);

        if snake_positions.contains(&new_head) {
            return distance; // the higher the distance to a body part, the higher the score
        }
    }

    // the lower the distance to an apple, the higher the score
    board_dim.h as usize + board_dim.v as usize - distance
}

impl Controller for Competitor1 {
    fn next_dir(
        &mut self,
        snake_body: &SnakeBody,
        other_snakes: OtherSnakes,
        apples: &[Apple],
        board_dim: HexDim,
    ) -> Option<Dir> {
        // all turns
        let available_directions: Vec<_> = once(snake_body.dir)
            .chain(snake_body.dir.blunt_turns().iter().copied())
            .chain(snake_body.dir.sharp_turns().iter().copied())
            .collect();

        // only blunt turns
        // let available_directions: Vec<_> = once(snake_body.dir)
        //     .chain(snake_body.dir.blunt_turns().iter().copied())
        //     .collect();

        // only sharp turns
        // let available_directions: Vec<_> = once(snake_body.dir)
        //     .chain(snake_body.dir.sharp_turns().iter().copied())
        //     .collect();

        let apple_positions: Vec<_> = apples.iter().map(|a| a.pos).collect();
        let snake_positions: Vec<_> = once(snake_body)
            .chain(other_snakes.iter_bodies())
            .flat_map(|b| b.cells.iter().map(|h| h.pos))
            .collect();

        let new_dir = available_directions
            .iter()
            .max_by_key(|&&dir| {
                dir_score(
                    snake_body.cells[0].pos,
                    dir,
                    board_dim,
                    &snake_positions,
                    &apple_positions,
                )
            })
            .copied();

        new_dir
    }
}