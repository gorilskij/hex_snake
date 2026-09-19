use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::Dir;
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::snake_control::Controller;
use crate::view::snakes::Snakes;

#[allow(unused_macros)]
macro_rules! move_sequence {
    (@ turn($dir:expr) ) => {
        crate::app::snake::controller::SimMove::Turn($dir)
    };
    (@ wait($t:expr) ) => {
        crate::app::snake::controller::SimMove::Wait($t)
    };
    [ $( $action:tt ( $( $inner:tt )* ) ),* $(,)? ] => {
        vec![$(
            move_sequence!(@ $action( $( $inner )* ))
        ),*]
    };
}

#[derive(Copy, Clone, Debug)]
pub enum Move {
    Turn(Dir),
    Wait(usize),
}

/// Plays a fixed sequence of moves on repeat, one step per cell.
pub struct Programmed {
    move_sequence: Vec<Move>,
    /// Direction before the sequence's first turn
    start_dir: Dir,
    dir: Dir,
    next_move_idx: usize,
    wait: usize,
    /// Cells into the current cycle of the sequence
    position: usize,
}

impl Programmed {
    pub fn new(move_sequence: Vec<Move>, start_dir: Dir) -> Self {
        Self {
            move_sequence,
            start_dir,
            dir: start_dir,
            next_move_idx: 0,
            wait: 0,
            position: 0,
        }
    }

    /// How many cells one cycle of the sequence takes: a turn is one cell, a
    /// wait as many cells as it lasts.
    fn cycle_len(&self) -> usize {
        self.move_sequence
            .iter()
            .map(|m| match *m {
                Move::Turn(_) => 1,
                Move::Wait(wait) => wait,
            })
            .sum()
    }

    /// Advance the sequence by one cell
    fn step(&mut self) -> Option<Dir> {
        if self.wait > 0 {
            self.wait -= 1;
        } else {
            match *self.move_sequence.get(self.next_move_idx)? {
                Move::Wait(wait) => self.wait = wait - 1,
                Move::Turn(new_dir) => self.dir = new_dir,
            };

            self.next_move_idx += 1;
            self.next_move_idx %= self.move_sequence.len();
        }

        self.position = (self.position + 1) % self.cycle_len();
        Some(self.dir)
    }
}

impl Controller for Programmed {
    fn next_dir(
        &mut self,
        _: &mut Body,
        _: Option<&Knowledge>,
        _: &dyn Snakes,
        _: &[Apple],
        _: &GameContext,
    ) -> Option<Dir> {
        self.step()
    }

    fn reset(&mut self, dir: Dir) {
        self.start_dir = dir;
        self.set_schedule_position(0);
    }

    fn schedule_position(&self) -> Option<usize> {
        Some(self.position)
    }

    /// Replays the sequence from its start, so the state is exactly what
    /// `position` steps would have left behind (wrapped to one cycle).
    fn set_schedule_position(&mut self, position: usize) {
        self.dir = self.start_dir;
        self.next_move_idx = 0;
        self.wait = 0;
        self.position = 0;
        if self.move_sequence.is_empty() {
            return;
        }
        for _ in 0..position % self.cycle_len() {
            self.step();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Dir::*;

    fn pattern() -> Programmed {
        Programmed::new(vec![Move::Turn(U), Move::Wait(2), Move::Turn(Ur), Move::Wait(3)], D)
    }

    #[test]
    fn position_counts_cells_and_wraps() {
        let mut p = pattern();
        assert_eq!(p.cycle_len(), 7);
        for i in 1..=10 {
            p.step();
            assert_eq!(p.schedule_position(), Some(i % 7));
        }
    }

    #[test]
    fn set_position_matches_stepping() {
        for n in 0..20 {
            let mut stepped = pattern();
            let mut expected = vec![];
            for _ in 0..n {
                stepped.step();
            }
            for _ in 0..10 {
                expected.push(stepped.step());
            }

            let mut jumped = pattern();
            jumped.set_schedule_position(n);
            let actual: Vec<_> = (0..10).map(|_| jumped.step()).collect();
            assert_eq!(actual, expected, "after jumping to {n}");
        }
    }
}
