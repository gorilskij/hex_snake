use crate::{
    app::{
        keyboard_control::ControlSetup,
        screen::game::Apple,
        snake::{
            controller::{
                a_star::AStar,
                competitor1::Competitor1,
                competitor2::Competitor2,
                keyboard::Keyboard,
                keyboard_clock::KeyboardClock,
                killer::Killer,
                programmed::{Move, Programmed},
            },
            utils::OtherSnakes,
            Body,
        },
    },
    basic::{Dir, Dir12, HexDim, Side},
    partial_min_max::{partial_max, partial_min},
};
use ggez::event::KeyCode;
use itertools::{repeat_n, Itertools};
use std::{collections::VecDeque, f32::consts::TAU};

mod a_star;
mod competitor1;
mod competitor2;
mod keyboard;
mod keyboard_clock;
mod killer;
pub mod programmed;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum ControllerTemplate {
    Keyboard(ControlSetup),
    KeyboardClock,
    Programmed(Vec<Move>),
    Competitor1,
    Competitor2,
    Killer,
    AStar,
}

pub trait Controller {
    // NOTE: there is a difference between returning None and the same dir
    //  returning None will cause the snake to query again on the
    //  next graphics frame, otherwise it will wait until the next game frame
    fn next_dir(
        &mut self,
        body: &mut Body,
        other_snakes: OtherSnakes,
        apples: &[Apple],
        board_dim: HexDim,
    ) -> Option<Dir>;

    fn reset(&mut self, _dir: Dir) {}

    fn key_pressed(&mut self, _key: KeyCode) {}
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
impl ControllerTemplate {
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
            iter.interleave(repeat_n(Move::Wait(side_len), 12))
                .collect()
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
    pub fn into_controller(self, start_dir: Dir) -> Box<dyn Controller> {
        match self {
            ControllerTemplate::Keyboard(control_setup) => Box::new(Keyboard {
                controls: control_setup.into(),
                control_queue: VecDeque::with_capacity(Keyboard::CTRL_QUEUE_LIMIT),
                dir: start_dir,
            }),
            ControllerTemplate::KeyboardClock => Box::new(KeyboardClock {
                dir: Dir12::Single(start_dir),
                alternation: false,
                next_dir: None,
            }),
            ControllerTemplate::Programmed(move_sequence) => Box::new(Programmed {
                move_sequence,
                dir: start_dir,
                next_move_idx: 0,
                wait: 0,
            }),
            ControllerTemplate::Competitor1 => Box::new(Competitor1),
            ControllerTemplate::Competitor2 => Box::new(Competitor2 {
                dir_state: false,
                target_apple: None,
                frames_since_update: 0,
            }),
            ControllerTemplate::Killer => Box::new(Killer),
            ControllerTemplate::AStar => Box::new(AStar {
                target: None,
                path: vec![],
                steps_since_update: 0,
            }),
        }
    }
}

// useful to multiple ai controllers
fn angle_distance(a1: f32, a2: f32) -> f32 {
    let d1 = (a1 - a2).abs();
    // add tau to the smaller of the two angles and consider that distance as well
    let b1 = partial_min(a1, a2).unwrap() + TAU;
    let b2 = partial_max(a1, a2).unwrap();
    let d2 = (b1 - b2).abs();
    partial_min(d1, d2).unwrap()
}
