use rand::distributions::uniform::SampleRange;
use rand::Rng;

use crate::apple::Apple;
use crate::basic::{HexDim, HexPoint};
use crate::snake::Snake;

pub fn get_occupied_cells(snakes: &[Snake], apples: &[Apple]) -> Vec<HexPoint> {
    // upper bound
    let max_occupied_cells = snakes.iter().map(|snake| snake.body.visible_len()).sum::<usize>() + apples.len();
    let mut occupied_cells = Vec::with_capacity(max_occupied_cells);
    occupied_cells.extend(apples.iter().map(|apple| apple.pos));
    for snake in snakes {
        occupied_cells.extend(snake.body.segments.iter().map(|hex| hex.pos));
    }
    occupied_cells.sort_unstable();
    occupied_cells.dedup();
    occupied_cells
}

pub fn random_free_spot(occupied_cells: &[HexPoint], board_dim: HexDim, rng: &mut impl Rng) -> Option<HexPoint> {
    let free_spaces = (board_dim.h * board_dim.v) as usize - occupied_cells.len();
    if free_spaces == 0 {
        return None;
    }

    let mut new_idx = (0..free_spaces).sample_single(rng);
    for HexPoint { h, v } in occupied_cells {
        let idx = (v * board_dim.h + h) as usize;
        if idx <= new_idx {
            new_idx += 1;
        }
    }

    assert!(new_idx < (board_dim.h * board_dim.v) as usize);
    Some(HexPoint {
        h: new_idx as isize % board_dim.h,
        v: new_idx as isize / board_dim.h,
    })
}
