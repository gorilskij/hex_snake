use ggez::input::keyboard::KeyCode;
use ggez::Context;
use itertools::{repeat_n, Itertools};
use programmed::Move;

use crate::app::fps_control::FpsContext;
use crate::app::game_context::GameContext;
use crate::app::keyboard_control::ControlSetup;
use crate::apple::Apple;
use crate::basic::{Dir, Dir12, Side};
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::snake_control::pathfinder::Path;
use crate::view::snakes::Snakes;

mod algorithm;
mod keyboard;
mod keyboard_clock;
mod killer;
mod mouse;
pub mod pathfinder;
mod programmed;
mod rain;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Template {
    Keyboard {
        control_setup: ControlSetup,
        knowledge: Knowledge,
    },
    KeyboardClock,
    Mouse,
    Programmed(Vec<Move>),
    Killer,
    Algorithm(pathfinder::Template),
    Rain,
}

pub trait Controller {
    // NOTE: there is a difference between returning None and the same dir
    //  returning None will cause the snake to query again on the
    //  next graphics frame, otherwise it will wait until the next game frame
    fn next_dir(
        &mut self,
        body: &mut Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
        ftx: &FpsContext,
        ctx: &Context,
    ) -> Option<Dir>;

    // only implemented for autopilot-like controllers
    fn get_path(
        &mut self,
        _body: &Body,
        _knowledge: Option<&Knowledge>,
        _other_snakes: &dyn Snakes,
        _apples: &[Apple],
        _gtx: &GameContext,
    ) -> Option<&Path> {
        None
    }

    fn reset(&mut self, _dir: Dir) {}

    fn key_pressed(&mut self, _key: KeyCode) {}

    // TODO: deprecate
    fn knowledge(&self) -> Option<&Knowledge> {
        None
    }
}

// Group contiguous instances of Move::Wait together
fn simplify_pattern<I: IntoIterator<Item = Move>>(iter: I) -> impl Iterator<Item = Move> {
    iter.into_iter().peekable().batching(|it| match it.next() {
        None => None,
        e @ Some(Move::Turn(_)) => e,
        Some(Move::Wait(mut n)) => {
            while matches!(it.peek(), Some(Move::Wait(_))) {
                n += match it.next() {
                    Some(Move::Wait(m)) => m,
                    _ => unreachable!(),
                }
            }
            Some(Move::Wait(n))
        }
    })
}

#[allow(dead_code)]
impl Template {
    pub fn demo_hexagon_pattern(start_dir: Dir, side_len: usize) -> Self {
        let mut vec = Vec::with_capacity(12);
        for dir in Dir::iter_from(start_dir) {
            vec.push(Move::Turn(dir));
            if side_len > 0 {
                vec.push(Move::Wait(side_len));
            }
        }
        Self::Programmed(vec)
    }

    pub fn demo_triangle_pattern(side_len: usize, pointing_towards: Side) -> Self {
        let mut vec = Vec::with_capacity(6);
        let mut dir = Dir::U;
        for _ in 0..3 {
            vec.push(Move::Turn(dir));
            if side_len > 0 {
                vec.push(Move::Wait(side_len));
            }
            dir = match pointing_towards {
                Side::Left => dir - 2,
                Side::Right => dir + 2,
            };
        }
        Self::Programmed(vec)
    }

    // assume starting at left side of infinity symbol, going up, clockwise
    pub fn demo_infinity_pattern(side_len: usize) -> Self {
        use Dir::*;

        const DIRECTIONS: [Dir; 12] = [U, Ur, Dr, D, Dr, Ur, U, Ul, Dl, D, Dl, Ul];
        let iter = DIRECTIONS.into_iter().map(Move::Turn);
        Self::Programmed(if side_len == 0 {
            iter.collect()
        } else {
            iter.interleave(repeat_n(Move::Wait(side_len), 12)).collect()
        })
    }

    // assume starting at top-left of 8 symbol, going up, clockwise
    pub fn demo_8_pattern(side_len: usize) -> Self {
        use Dir::*;
        use Move::*;

        let vec = vec![
            Turn(U),
            Turn(Ur),
            Turn(Dr),
            Turn(D),
            Turn(Dl),
            Wait(1),
            Turn(D),
            Turn(Dr),
            Turn(Ur),
            Turn(U),
            Turn(Ul),
            Wait(1),
        ];
        Self::Programmed(if side_len == 0 {
            vec
        } else {
            let iter = vec.into_iter().interleave(repeat_n(Wait(side_len), 12));
            simplify_pattern(iter).collect()
        })
    }

    // TODO: remove start_dir
    pub fn into_controller(self, start_dir: Dir) -> Box<dyn Controller + Send + Sync> {
        // use crate::snake_control::a_star::AStar;
        use algorithm::Algorithm;
        use keyboard::Keyboard;
        use keyboard_clock::KeyboardClock;
        use killer::Killer;
        use mouse::Mouse;
        use programmed::Programmed;
        use rain::Rain;

        match self {
            Template::Keyboard { control_setup, knowledge } => {
                Box::new(Keyboard::new(control_setup, start_dir, knowledge))
            }
            Template::KeyboardClock => Box::new(KeyboardClock {
                dir: Dir12::Single(start_dir),
                alternation: false,
                next_dir: None,
            }),
            Template::Mouse => Box::new(Mouse),
            Template::Programmed(move_sequence) => Box::new(Programmed {
                move_sequence,
                dir: start_dir,
                next_move_idx: 0,
                wait: 0,
            }),
            Template::Killer => Box::new(Killer),
            Template::Algorithm(template) => Box::new(Algorithm {
                pathfinder: template.into_pathfinder(start_dir),
                path: None,
            }),
            Template::Rain => Box::new(Rain),
        }
    }
}
