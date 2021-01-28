use crate::app::{
    game::{Apple, CellDim},
    hex::{Dir, HexDim, HexPoint},
    keyboard_control::{ControlSetup, Controls},
    snake::{Segment, Snake, SnakeBody, SnakeType},
};
use ggez::event::KeyCode;
use std::{cmp::Ordering, collections::VecDeque, iter::once};

#[derive(Clone)]
pub enum SnakeControllerTemplate {
    PlayerController(ControlSetup),
    PlayerController12,
    DemoController(Vec<SimMove>),
    CompetitorAI,
    KillerAI,
}

#[derive(Copy, Clone)]
pub struct OtherSnakes<'a>(pub &'a [Snake], pub &'a [Snake]);

impl OtherSnakes<'_> {
    pub fn iter_snakes(&self) -> impl Iterator<Item = &Snake> {
        self.0.iter().chain(self.1.iter())
    }

    pub fn iter_bodies(&self) -> impl Iterator<Item = &SnakeBody> {
        self.iter_snakes().map(|Snake { body, .. }| body)
    }
}

pub trait SnakeController {
    fn next_dir(
        &mut self,
        snake_body: &SnakeBody,
        other_snakes: OtherSnakes,
        apples: &[Apple],
        board_dim: HexDim,
    ) -> Option<Dir>;

    fn reset(&mut self) {}

    fn key_pressed(&mut self, _key: KeyCode) {}
}

impl SnakeControllerTemplate {
    pub fn into_controller(self, initial_dir: Dir) -> Box<dyn SnakeController> {
        match self {
            SnakeControllerTemplate::PlayerController(control_setup) => {
                Box::new(PlayerController {
                    controls: control_setup.into(),
                    control_queue: VecDeque::with_capacity(PlayerController::CTRL_QUEUE_LIMIT),
                    dir: initial_dir,
                })
            }
            SnakeControllerTemplate::PlayerController12 => Box::new(PlayerController12 {
                dir: Dir12::Single(initial_dir),
                alternation: false,
                next_dir: None,
            }),
            SnakeControllerTemplate::DemoController(move_sequence) => Box::new(DemoController {
                move_sequence,
                next_move_idx: 0,
                wait: 0,
            }),
            SnakeControllerTemplate::CompetitorAI => Box::new(CompetitorAI),
            SnakeControllerTemplate::KillerAI => Box::new(KillerAI),
        }
    }
}

pub struct PlayerController {
    controls: Controls,
    control_queue: VecDeque<Dir>,
    dir: Dir,
}

impl PlayerController {
    const CTRL_QUEUE_LIMIT: usize = 3;
}

impl SnakeController for PlayerController {
    fn next_dir(&mut self, _: &SnakeBody, _: OtherSnakes, _: &[Apple], _: HexPoint) -> Option<Dir> {
        if let Some(queue_dir) = self.control_queue.pop_front() {
            self.dir = queue_dir;
            Some(self.dir)
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

        // deny 180deg turns
        if self.control_queue.is_empty() && self.dir != -new_dir
            || !self.control_queue.is_empty()
                && self.control_queue.len() < Self::CTRL_QUEUE_LIMIT
                && new_dir != -self.control_queue[self.control_queue.len() - 1]
        {
            self.control_queue.push_back(new_dir)
        }
    }
}

// with 6 simulated directions between the 6 normal ones
#[derive(Copy, Clone)]
enum Dir12 {
    Single(Dir),
    Combined(Dir, Dir),
}

impl Dir12 {
    const ORDER: &'static [Dir12] = &[
        Dir12::Single(Dir::U),
        Dir12::Combined(Dir::U, Dir::UR),
        Dir12::Single(Dir::UR),
        Dir12::Combined(Dir::UR, Dir::DR),
        Dir12::Single(Dir::DR),
        Dir12::Combined(Dir::DR, Dir::D),
        Dir12::Single(Dir::D),
        Dir12::Combined(Dir::D, Dir::DL),
        Dir12::Single(Dir::DL),
        Dir12::Combined(Dir::DL, Dir::UL),
        Dir12::Single(Dir::UL),
        Dir12::Combined(Dir::UL, Dir::U),
    ];
}

impl PartialEq for Dir12 {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Single(d1), Self::Single(d2)) => d1 == d2,
            (Self::Combined(a1, b1), Self::Combined(a2, b2)) => {
                // order of a and b is irrelevant
                a1 == a2 && b1 == b2 || a1 == b2 && a2 == b1
            }
            _ => false,
        }
    }
}

impl Eq for Dir12 {}

// fairly hacky, doesn't interact well with teleportation
pub struct PlayerController12 {
    dir: Dir12,
    alternation: bool,
    next_dir: Option<Dir12>,
}

impl SnakeController for PlayerController12 {
    fn next_dir(&mut self, _: &SnakeBody, _: OtherSnakes, _: &[Apple], _: HexDim) -> Option<Dir> {
        if let Some(new_dir) = self.next_dir.take() {
            self.dir = new_dir;
            self.alternation = false;
        }
        match &mut self.dir {
            Dir12::Single(dir) => Some(*dir),
            Dir12::Combined(a, b) => {
                self.alternation = !self.alternation;
                match self.alternation {
                    true => Some(*a),
                    false => Some(*b),
                }
            }
        }
    }

    fn reset(&mut self) {
        self.next_dir = None;
    }

    fn key_pressed(&mut self, key: KeyCode) {
        match key {
            KeyCode::Left => {
                let mut current_idx = Dir12::ORDER
                    .iter()
                    .position(|d12| *d12 == self.dir)
                    .unwrap();
                current_idx = (current_idx + 12 - 1) % 12;
                self.dir = Dir12::ORDER[current_idx];
            }
            KeyCode::Right => {
                let mut current_idx = Dir12::ORDER
                    .iter()
                    .position(|d12| *d12 == self.dir)
                    .unwrap();
                current_idx = (current_idx + 1) % 12;
                self.dir = Dir12::ORDER[current_idx];
            }
            _ => (),
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
    fn next_dir(&mut self, _: &SnakeBody, _: OtherSnakes, _: &[Apple], _: HexPoint) -> Option<Dir> {
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

// competes for apples
pub struct CompetitorAI;

// old way (might be more efficient for sparse boards, not tested)
// fn dir_score(
//     head: HexPos,
//     dir: Dir,
//     board_dim: HexDim,
//     snake_body: &SnakeBody,
//     other_snakes: OtherSnakes,
//     apples: &[Apple],
// ) -> usize {
//     let mut distance = 0;
//     let mut new_head = head;
//     while !apples.iter().any(|Apple { pos, .. }| pos == &new_head) {
//         distance += 1;
//         new_head = new_head.wrapping_translate(dir, 1, board_dim);
//
//         for body in once(snake_body).chain(other_snakes.iter_bodies()) {
//             if body.cells.iter().any(|Hex { pos, .. }| pos == &new_head) {
//                 return distance; // the higher the distance to a body part, the higher the score
//             }
//         }
//     }
//     // println!("for dir {:?}, dist: {}", dir, distance);
//     // the lower the distance to an apple, the higher the score
//     board_dim.h as usize + board_dim.v as usize - distance
// }

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
    // println!("for dir {:?}, dist: {}", dir, distance);
    // the lower the distance to an apple, the higher the score
    board_dim.h as usize + board_dim.v as usize - distance
}

impl SnakeController for CompetitorAI {
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

// tries to kill player
struct KillerAI;

#[derive(PartialOrd, PartialEq)]
struct TotalF32(f32);

impl Eq for TotalF32 {}

#[allow(clippy::derive_ord_xor_partial_ord)] // sound because Ord just forwards to PartialOrd
impl Ord for TotalF32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn distance_to_snake(
    dir: Dir,
    snake_body: &SnakeBody,
    other_snakes: OtherSnakes,
    board_dim: HexDim,
) -> usize {
    let mut distance = 0;
    let mut new_head = snake_body.cells[0].pos;
    // guaranteed to terminate whenever the head reaches itself again
    loop {
        distance += 1;
        new_head = new_head.wrapping_translate(dir, 1, board_dim);

        for body in once(snake_body).chain(other_snakes.iter_bodies()) {
            if body
                .cells
                .iter()
                .any(|Segment { pos, .. }| pos == &new_head)
            {
                return distance;
            }
        }
    }
}

// potential alternative to searching for the closest angle
// would still require searching through a list so not an improvement..
// fn round_angle_to_closest_dir(angle: f32) -> Dir {
//     use std::f32::consts::{FRAC_PI_3, FRAC_PI_6};
//     let rounded_angle = (angle / FRAC_PI_3).floor() * FRAC_PI_3 + FRAC_PI_6;
// }

fn rough_direction(
    from: HexPoint,
    to: HexPoint,
    snake_body: &SnakeBody,
    other_snakes: OtherSnakes,
    board_dim: HexDim,
) -> Option<Dir> {
    use std::f32::consts::PI;
    use Dir::*;

    const TWO_PI: f32 = 2. * PI;

    // dy is scaled to convert from 'hex' coordinates to approximate cartesian coordinates
    let CellDim { sin, .. } = CellDim::from(1.);
    let dx = (to.h - from.h) as f32;
    let dy = -(to.v - from.v) as f32 / (2. * sin);
    let angle = (dy.atan2(dx) + TWO_PI) % TWO_PI;

    const DIR_ANGLES: [(Dir, f32); 6] = [
        (UR, 1. / 6. * PI),
        (U, 3. / 6. * PI),
        (UL, 5. / 6. * PI),
        (DL, 7. / 6. * PI),
        (D, 9. / 6. * PI),
        (DR, 11. / 6. * PI),
    ];

    // this could probably be done with math
    DIR_ANGLES
        .iter()
        .copied()
        .filter(|(d, _)| *d != -snake_body.dir)
        .filter(|(d, _)| distance_to_snake(*d, snake_body, other_snakes, board_dim) > 1)
        .min_by_key(|(_, a)| TotalF32((a - angle).abs()))
        .take()
        .map(|(d, _)| d)
}

impl SnakeController for KillerAI {
    // TODO: this gets called when the snake should be dying
    //  e.g. when the last player dies
    fn next_dir(
        &mut self,
        snake_body: &SnakeBody,
        other_snakes: OtherSnakes,
        _apples: &[Apple],
        board_dim: HexDim,
    ) -> Option<Dir> {
        let player_snake = other_snakes
            .iter_snakes()
            .filter(|s| s.snake_type == SnakeType::PlayerSnake)
            .min_by_key(|s| s.head().pos.distance_to(snake_body.cells[0].pos))
            .expect("no player snake found");

        let mut target = player_snake.head().pos;
        // how many cells ahead of the player to target
        for _ in 0..1 {
            target = target.wrapping_translate(player_snake.dir(), 1, board_dim);
        }
        rough_direction(
            snake_body.cells[0].pos,
            target,
            snake_body,
            other_snakes,
            board_dim,
        )
    }
}
