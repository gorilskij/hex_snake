use rand::distributions::uniform::SampleRange;
use rand::Rng;

use crate::apple::Apple;
use crate::basic::{CellDim, Dir, HexDim, HexPoint, Point};
use crate::snake::{self, Snake};

/// How close to a player snake's head new things avoid spawning
const PLAYER_HEAD_NO_SPAWN_RADIUS: usize = 7;

/// How far one step in `dir` moves in cartesian space. This is constant per
/// direction, unlike the hex coordinate delta, which depends on column parity.
///
/// Stepping from a cell is how geometry stays right across a board edge: two
/// cells either side of the wrap are a whole board apart in board coordinates
/// but one step apart on the board.
pub fn cartesian_step(dir: Dir, cell_dim: CellDim) -> Point {
    let CellDim { side, sin, cos } = cell_dim;
    let dx = side + cos;
    match dir {
        Dir::U => Point { x: 0., y: -2. * sin },
        Dir::D => Point { x: 0., y: 2. * sin },
        Dir::Ur => Point { x: dx, y: -sin },
        Dir::Ul => Point { x: -dx, y: -sin },
        Dir::Dr => Point { x: dx, y: sin },
        Dir::Dl => Point { x: -dx, y: sin },
    }
}

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

/// The occupied cells plus the neighborhood of every player snake's head, so
/// that new things don't appear right in front of a player. Sorted, for
/// [`random_free_spot`].
pub fn occupied_or_near_players(snakes: &[Snake], apples: &[Apple], board_dim: HexDim) -> Vec<HexPoint> {
    let mut cells = get_occupied_cells(snakes, apples);
    for snake in snakes.iter().filter(|s| s.snake_type == snake::Type::Player) {
        cells.extend_from_slice(&snake.reachable(PLAYER_HEAD_NO_SPAWN_RADIUS, board_dim));
    }
    cells.sort_unstable();
    cells.dedup();
    cells
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

#[cfg(test)]
mod tests {
    use super::*;

    const CELL_DIM: CellDim = CellDim { side: 50., sin: 43.30127, cos: 25. };

    /// The cartesian step must agree with the cells' own cartesian positions,
    /// for both column parities (the hex coordinate delta differs between them,
    /// the cartesian one does not).
    #[test]
    fn cartesian_step_matches_cell_positions() {
        for h in [4, 5] {
            let cell = HexPoint { h, v: 4 };
            for dir in Dir::iter() {
                let expected = cell.translate(dir, 1).to_cartesian(CELL_DIM) - cell.to_cartesian(CELL_DIM);
                let step = cartesian_step(dir, CELL_DIM);
                assert!(
                    (step.x - expected.x).abs() < 0.01 && (step.y - expected.y).abs() < 0.01,
                    "{dir:?} from h={h}: step {step:?}, expected {expected:?}",
                );
            }
        }
    }

    /// Stepping is what keeps geometry right across a board edge: the wrapped
    /// cell is one step away, while its board position is a whole board away.
    #[test]
    fn stepping_beats_board_position_across_the_wrap() {
        const BOARD: HexDim = HexPoint { h: 10, v: 10 };
        let edge = HexPoint { h: 0, v: 5 };

        let wrapped = edge.wrapping_translate(Dir::Ul, 1, BOARD);
        assert_eq!(wrapped.h, BOARD.h - 1, "the step should have wrapped");

        let origin = edge.to_cartesian(CELL_DIM);
        let stepped = (origin + cartesian_step(Dir::Ul, CELL_DIM) - origin).magnitude();
        let board_space = (wrapped.to_cartesian(CELL_DIM) - origin).magnitude();

        assert!(
            board_space > 5. * stepped,
            "board space puts the wrapped neighbour {board_space} away, stepping puts it {stepped}",
        );
    }
}
