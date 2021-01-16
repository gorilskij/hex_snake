use crate::app::{
    control::{ControlSetup, Controls},
    game::Apple,
    hex::{Dir, Hex, HexPos},
    snake::{Snake, SnakeBody},
};
use ggez::event::KeyCode;
use std::{collections::VecDeque, iter::once};

#[derive(Clone)]
pub enum SnakeControllerTemplate {
    PlayerController(ControlSetup),
    DemoController(Vec<SimMove>),
    SnakeAI,
}

#[derive(Copy, Clone)]
pub struct OtherBodies<'a>(pub &'a [Snake], pub &'a [Snake]);

impl OtherBodies<'_> {
    pub fn iter(&self) -> impl Iterator<Item = &SnakeBody> {
        self.0
            .iter()
            .chain(self.1.iter())
            .map(|Snake { body, .. }| body)
    }
}

pub trait SnakeController {
    fn next_dir(
        &mut self,
        snake_body: &SnakeBody,
        other_bodies: OtherBodies,
        apples: &[Apple],
        board_dim: HexPos,
    ) -> Option<Dir>;

    fn reset(&mut self) {}

    fn key_pressed(&mut self, _key: KeyCode) {}
}

impl From<SnakeControllerTemplate> for Box<dyn SnakeController> {
    fn from(template: SnakeControllerTemplate) -> Self {
        match template {
            SnakeControllerTemplate::PlayerController(control_setup) => {
                Box::new(PlayerController {
                    controls: control_setup.into(),
                    control_queue: VecDeque::with_capacity(PlayerController::CTRL_QUEUE_LIMIT),
                    dir: None,
                })
            }
            SnakeControllerTemplate::DemoController(move_sequence) => Box::new(DemoController {
                move_sequence,
                next_move_idx: 0,
                wait: 0,
            }),
            SnakeControllerTemplate::SnakeAI => Box::new(SnakeAI),
        }
    }
}

pub struct PlayerController {
    controls: Controls,
    control_queue: VecDeque<Dir>,
    dir: Option<Dir>,
}

impl PlayerController {
    const CTRL_QUEUE_LIMIT: usize = 3;
}

impl SnakeController for PlayerController {
    fn next_dir(&mut self, _: &SnakeBody, _: OtherBodies, _: &[Apple], _: HexPos) -> Option<Dir> {
        if let Some(queue_dir) = self.control_queue.pop_front() {
            self.dir = Some(queue_dir);
            self.dir
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.control_queue.clear()
    }

    fn key_pressed(&mut self, key: KeyCode) {
        use Dir::*;
        let new_dir = match key {
            k if k == self.controls.u => U,
            k if k == self.controls.d => D,
            k if k == self.controls.ul => UL,
            k if k == self.controls.ur => UR,
            k if k == self.controls.dl => DL,
            k if k == self.controls.dr => DR,
            _ => return,
        };

        if self.control_queue.is_empty() && self.dir.map(|dir| new_dir != -dir).unwrap_or(true)
            || !self.control_queue.is_empty()
                && self.control_queue.len() < Self::CTRL_QUEUE_LIMIT
                && new_dir != -self.control_queue[self.control_queue.len() - 1]
        {
            self.control_queue.push_back(new_dir)
        }
    }
}

#[derive(Copy, Clone)]
pub enum SimMove {
    Move(Dir),
    Wait(usize),
}

pub struct DemoController {
    move_sequence: Vec<SimMove>,
    next_move_idx: usize,
    wait: usize,
}

impl SnakeController for DemoController {
    fn next_dir(&mut self, _: &SnakeBody, _: OtherBodies, _: &[Apple], _: HexPos) -> Option<Dir> {
        if self.wait > 0 {
            self.wait -= 1;
            None
        } else {
            let new_dir = match self.move_sequence[self.next_move_idx] {
                SimMove::Wait(wait) => {
                    self.wait = wait;
                    None
                }
                SimMove::Move(new_dir) => Some(new_dir),
            };

            self.next_move_idx += 1;
            self.next_move_idx %= self.move_sequence.len();
            new_dir
        }
    }

    fn reset(&mut self) {
        self.next_move_idx = 0;
        self.wait = 0;
    }
}

pub struct SnakeAI;

fn dir_score(
    head: HexPos,
    dir: Dir,
    board_dim: HexPos,
    snake_body: &SnakeBody,
    other_bodies: OtherBodies,
    apples: &[Apple],
) -> usize {
    let mut distance = 0;
    let mut new_head = head;
    while !apples.iter().any(|Apple { pos, .. }| pos == &new_head) {
        distance += 1;
        new_head.step_and_teleport(dir, board_dim);

        for snake in once(snake_body).chain(other_bodies.iter()) {
            if snake.body.iter().any(|Hex { pos, .. }| pos == &new_head) {
                return distance; // the higher the distance to a body part, the higher the score
            }
        }
    }
    // println!("for dir {:?}, dist: {}", dir, distance);
    // the lower the distance to an apple, the higher the score
    board_dim.h as usize + board_dim.v as usize - distance
}

impl SnakeController for SnakeAI {
    fn next_dir(
        &mut self,
        snake_body: &SnakeBody,
        other_bodies: OtherBodies,
        apples: &[Apple],
        board_dim: HexPos,
    ) -> Option<Dir> {
        use Dir::*;
        let available_directions: Vec<_> = [UL, U, UR, DL, D, DR]
            .iter()
            .filter(|&&d| d != -snake_body.dir)
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

        let new_dir = available_directions
            .iter()
            .max_by_key(|&&dir| {
                dir_score(
                    snake_body.body[0].pos,
                    dir,
                    board_dim,
                    &snake_body,
                    other_bodies,
                    apples,
                )
            })
            .copied();

        // if let Some(dir) = new_dir {
        //     println!("new: {:?}", dir)
        // }
        new_dir
    }
}
