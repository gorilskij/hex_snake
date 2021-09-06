use crate::{
    app::{
        keyboard_control::ControlSetup,
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
            Segment, Snake, Body,
        },
    },
    basic::{Dir, Dir12, HexDim, Side},
    partial_min_max::{partial_max, partial_min},
};
use ggez::event::KeyCode;
use std::{collections::VecDeque, f32::consts::TAU};
use crate::app::snake::utils::OtherSnakes;
use crate::app::screen::game::Apple;


mod a_star;
mod competitor1;
mod competitor2;
mod keyboard;
mod keyboard_clock;
mod killer;
pub mod programmed;

#[allow(dead_code)]
#[derive(Clone)]
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

#[allow(dead_code)]
impl ControllerTemplate {
    pub fn demo_hexagon_pattern(side_len: usize) -> Self {
        let mut vec = Vec::with_capacity(12);
        for dir in Dir::iter() {
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

    // TODO: remove initial_dir
    pub fn into_controller(self, initial_dir: Dir) -> Box<dyn Controller> {
        match self {
            ControllerTemplate::Keyboard(control_setup) => Box::new(Keyboard {
                controls: control_setup.into(),
                control_queue: VecDeque::with_capacity(Keyboard::CTRL_QUEUE_LIMIT),
                dir: initial_dir,
            }),
            ControllerTemplate::KeyboardClock => Box::new(KeyboardClock {
                dir: Dir12::Single(initial_dir),
                alternation: false,
                next_dir: None,
            }),
            ControllerTemplate::Programmed(move_sequence) => Box::new(Programmed {
                move_sequence,
                dir: initial_dir,
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
    // add 2pi to the smaller of the two angles and consider that distance as well
    let b1 = partial_min(a1, a2).unwrap() + TAU;
    let b2 = partial_max(a1, a2).unwrap();
    let d2 = (b1 - b2).abs();
    partial_min(d1, d2).unwrap()
}
