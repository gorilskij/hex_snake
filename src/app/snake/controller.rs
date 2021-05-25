use crate::{
    app::{
        game::Apple,
        keyboard_control::{ControlSetup, Controls},
        snake::{Segment, Snake, SnakeBody, SnakeType},
    },
    basic::{CellDim, Dir, Dir12, HexDim, HexPoint, Side},
};
use ggez::event::KeyCode;
use itertools::Itertools;
use std::{
    cmp::Ordering,
    collections::{HashSet, VecDeque},
    f32::consts::PI,
    iter::once,
};

// because Iterator::min_by_key requires Ord
#[derive(PartialEq)]
#[repr(transparent)]
struct TotalF32(f32);

impl Eq for TotalF32 {}

impl PartialOrd for TotalF32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for TotalF32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone)]
pub enum ControllerTemplate {
    PlayerController(ControlSetup),
    PlayerController12,
    DemoController(Vec<SimMove>),
    CompetitorAI,
    CompetitorAI2,
    KillerAI,
    AStarAI,
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

    pub fn iter_segments(&self) -> impl Iterator<Item = &Segment> {
        self.iter_bodies().flat_map(|body| body.cells.iter())
    }

    pub fn contains(&self, point: HexPoint) -> bool {
        self.0
            .iter()
            .chain(self.1.iter())
            .any(|snake| snake.body.cells.iter().any(|segment| segment.pos == point))
    }
}

pub trait Controller {
    // NOTE: there is a difference between returning None and the same dir
    //  returning None will cause the snake to query again on the
    //  next graphics frame, otherwise it will wait until the next game frame
    fn next_dir(
        &mut self,
        snake_body: &SnakeBody,
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
            vec.push(SimMove::Turn(dir));
            if side_len > 0 {
                vec.push(SimMove::Wait(side_len));
            }
        }
        Self::DemoController(vec)
    }

    pub fn demo_triangle_pattern(side_len: usize, pointing_towards: Side) -> Self {
        let mut vec = Vec::with_capacity(6);
        let mut dir = Dir::U;
        for _ in 0..3 {
            vec.push(SimMove::Turn(dir));
            if side_len > 0 {
                vec.push(SimMove::Wait(side_len));
            }
            dir = match pointing_towards {
                Side::Left => dir - 2,
                Side::Right => dir + 2,
            };
        }
        Self::DemoController(vec)
    }

    // TODO: remove initial_dir
    pub fn into_controller(self, initial_dir: Dir) -> Box<dyn Controller> {
        match self {
            ControllerTemplate::PlayerController(control_setup) => Box::new(PlayerController {
                controls: control_setup.into(),
                control_queue: VecDeque::with_capacity(PlayerController::CTRL_QUEUE_LIMIT),
                dir: initial_dir,
            }),
            ControllerTemplate::PlayerController12 => Box::new(PlayerController12 {
                dir: Dir12::Single(initial_dir),
                alternation: false,
                next_dir: None,
            }),
            ControllerTemplate::DemoController(move_sequence) => Box::new(DemoController {
                move_sequence,
                dir: initial_dir,
                next_move_idx: 0,
                wait: 0,
            }),
            ControllerTemplate::CompetitorAI => Box::new(CompetitorAI),
            ControllerTemplate::CompetitorAI2 => Box::new(CompetitorAI2 {
                dir_state: false,
                target_apple: None,
                frames_since_update: 0,
            }),
            ControllerTemplate::KillerAI => Box::new(KillerAI),
            ControllerTemplate::AStarAI => Box::new(AStarAI {
                target: None,
                path: vec![],
                steps_since_update: 0,
            }),
        }
    }
}

struct PlayerController {
    controls: Controls,
    control_queue: VecDeque<Dir>,
    dir: Dir,
}

impl PlayerController {
    const CTRL_QUEUE_LIMIT: usize = 3;
}

impl Controller for PlayerController {
    fn next_dir(&mut self, _: &SnakeBody, _: OtherSnakes, _: &[Apple], _: HexDim) -> Option<Dir> {
        if let Some(queue_dir) = self.control_queue.pop_front() {
            self.dir = queue_dir;
            Some(self.dir)
        } else {
            None
        }
    }

    fn reset(&mut self, dir: Dir) {
        self.control_queue.clear();
        self.dir = dir;
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

// fairly hacky, doesn't interact well with teleportation
struct PlayerController12 {
    dir: Dir12,
    alternation: bool,
    next_dir: Option<Dir12>,
}

impl Controller for PlayerController12 {
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

    fn reset(&mut self, dir: Dir) {
        self.next_dir = None;
        self.dir = Dir12::Single(dir);
    }

    fn key_pressed(&mut self, key: KeyCode) {
        match key {
            KeyCode::Left => self.dir -= 1,
            KeyCode::Right => self.dir += 1,
            _ => (),
        }
    }
}

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
pub enum SimMove {
    Turn(Dir),
    Wait(usize),
}

struct DemoController {
    move_sequence: Vec<SimMove>,
    dir: Dir,
    next_move_idx: usize,
    wait: usize,
}

impl Controller for DemoController {
    fn next_dir(&mut self, _: &SnakeBody, _: OtherSnakes, _: &[Apple], _: HexPoint) -> Option<Dir> {
        if self.wait > 0 {
            self.wait -= 1;
        } else {
            match self.move_sequence[self.next_move_idx] {
                SimMove::Wait(wait) => self.wait = wait - 1,
                SimMove::Turn(new_dir) => self.dir = new_dir,
            };

            self.next_move_idx += 1;
            self.next_move_idx %= self.move_sequence.len();
        }

        Some(self.dir)
    }

    fn reset(&mut self, dir: Dir) {
        self.dir = dir;
        self.next_move_idx = 0;
        self.wait = 0;
    }
}

// competes for apples
struct CompetitorAI;

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

impl Controller for CompetitorAI {
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

struct CompetitorAI2 {
    dir_state: bool, // Dir12 flip-flop state
    target_apple: Option<HexPoint>,
    frames_since_update: usize,
}

impl CompetitorAI2 {
    const UPDATE_EVERY_N_FRAMES: usize = 20;
}

fn distance_to_snake(
    mut point: HexPoint,
    dir: Dir,
    snake_body: &SnakeBody,
    other_snakes: OtherSnakes,
    board_dim: HexDim,
    max_dist: Option<usize>, // if not within max_dist, returns max_dist
) -> usize {
    // guaranteed to terminate anyway whenever the head reaches itself again
    let upper_bound = max_dist.unwrap_or(usize::MAX);
    for distance in 1..=upper_bound {
        point = point.wrapping_translate(dir, 1, board_dim);

        for Segment { pos, .. } in snake_body.cells.iter().chain(other_snakes.iter_segments()) {
            if *pos == point {
                return distance;
            }
        }
    }
    upper_bound
}

// favors the first number in case comparison is not possible
fn partial_min<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        b
    } else {
        a
    }
}

// favors the first number in case comparison is not possible
fn partial_max<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        b
    } else {
        a
    }
}

fn angle_distance(a1: f32, a2: f32) -> f32 {
    let d1 = (a1 - a2).abs();
    // add 2pi to the smaller of the two angles and consider that distance as well
    let b1 = partial_min(a1, a2) + 2. * PI;
    let b2 = partial_max(a1, a2);
    let d2 = (b1 - b2).abs();
    partial_min(d1, d2)
}

impl Controller for CompetitorAI2 {
    fn next_dir(
        &mut self,
        snake_body: &SnakeBody,
        other_snakes: OtherSnakes,
        apples: &[Apple],
        _board_dim: HexDim,
    ) -> Option<Dir> {
        // this also sets the target apple on the first frame
        if self.frames_since_update % Self::UPDATE_EVERY_N_FRAMES == 0 {
            self.target_apple = None;
        }
        self.frames_since_update += 1;

        let head_pos = snake_body.cells[0].pos;
        if let Some(pos) = self.target_apple {
            if pos == head_pos {
                // apple eaten
                self.target_apple = None;
            }
        }

        let target_pos = match self.target_apple {
            None => {
                let closest_apple = apples
                    .iter()
                    .map(|apple| apple.pos)
                    .min_by_key(|pos| head_pos.manhattan_distance_to(*pos))?;
                self.target_apple = Some(closest_apple);
                closest_apple
            }
            Some(pos) => pos,
        };

        const TWO_PI: f32 = 2. * PI;
        let CellDim { side, sin, cos, .. } = CellDim::from(1.);

        let x_step = side + cos;
        let y_step = 2. * sin;

        let dh = target_pos.h - head_pos.h;
        let dv = target_pos.v - head_pos.v;

        let dx = dh as f32 / x_step;
        let dy = -dv as f32 / y_step; // convert to y going up
        let angle = (dy.atan2(dx) + TWO_PI) % TWO_PI;

        let mut forbidden_directions = snake_body
            .cells
            .iter()
            .chain(other_snakes.iter_segments())
            .map(|seg| seg.pos)
            .filter(|pos| pos.manhattan_distance_to(head_pos) == 1)
            .map(|pos| {
                head_pos
                    .dir_to(pos)
                    .unwrap_or_else(|| panic!("no direction from {:?} to {:?}", head_pos, pos))
            })
            .collect_vec();
        forbidden_directions.push(-snake_body.dir);

        // let dir_is_safe = |dir: Dir12| {
        //     if dir == Single(-snake_body.dir) {
        //         return false;
        //     }
        //     let translate_dir = dir.to_dir(self.dir_state);
        //     let new_head = head_pos.wrapping_translate(translate_dir, 1, board_dim);
        //     !forbidden_head_positions.contains(&new_head)
        // };

        // this could probably be done with math
        let new_dir = Dir12::ANGLES
            .iter()
            .copied()
            .map(|(d, a)| (d.to_dir(self.dir_state), a))
            .filter(|(d, _)| !forbidden_directions.contains(d))
            .min_by_key(|(_, a)| TotalF32(angle_distance(angle, *a)))
            .map(|(d, _)| d);

        // println!("preferred dir: {:?}", dir);
        // println!("target: {:?}", self.target_apple);

        self.dir_state = !self.dir_state;
        new_dir
    }
}

// tries to kill player
struct KillerAI;

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
    const TWO_PI: f32 = 2. * PI;

    // dy is scaled to convert from 'hex' coordinates to approximate cartesian coordinates
    let CellDim { sin, .. } = CellDim::from(1.);
    let dx = (to.h - from.h) as f32;
    let dy = -(to.v - from.v) as f32 / (2. * sin);
    let angle = (dy.atan2(dx) + TWO_PI) % TWO_PI;

    let head_pos = snake_body.cells[0].pos;

    // this could probably be done with math
    Dir::ANGLES
        .iter()
        .copied()
        .filter(|(d, _)| *d != -snake_body.dir)
        .filter(|(d, _)| {
            distance_to_snake(head_pos, *d, snake_body, other_snakes, board_dim, Some(2)) > 1
        })
        .min_by_key(|(_, a)| TotalF32(angle_distance(angle, *a)))
        .take()
        .map(|(d, _)| d)
}

impl Controller for KillerAI {
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
            .min_by_key(|s| s.head().pos.manhattan_distance_to(snake_body.cells[0].pos))
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

pub struct AStarAI {
    target: Option<HexPoint>,
    path: Vec<Dir>,
    steps_since_update: usize,
}

#[cfg(debug_assertions)]
pub static mut ETHEREAL_PATH: Option<Vec<HexPoint>> = None;

impl AStarAI {
    const UPDATE_EVERY_N_STEPS: usize = 1000;

    fn recalculate_target(&mut self, head: HexPoint, apples: &[Apple]) {
        let new_target = apples
            .iter()
            .map(|a| a.pos)
            .min_by_key(|p| head.manhattan_distance_to(*p));
        if new_target != self.target {
            self.target = new_target;
            self.path.clear();
        }
    }

    fn recalculate_path(
        &mut self,
        snake_body: &SnakeBody,
        other_snakes: OtherSnakes,
        board_dim: HexDim,
    ) {
        let target = match self.target {
            Some(p) => p,
            None => {
                self.path.clear();
                return;
            }
        };

        // A* search

        let head = snake_body[0].pos;

        let mut seen = HashSet::new();
        seen.insert(head);
        // the last node in each path is the newest
        let mut paths = vec![vec![head]];

        let mut forbidden_positions = snake_body
            .iter()
            .chain(other_snakes.iter_segments())
            .map(|seg| seg.pos)
            .collect::<HashSet<_>>();

        loop {
            // select which node to expand (f(x) is the length of the path)
            let expand_idx = paths.iter().position_min_by_key(|path| {
                path.len() - 1 + path.last().unwrap().manhattan_distance_to(target)
            });
            let expand_idx = match expand_idx {
                Some(idx) => idx,
                None => {
                    self.path.clear();
                    return;
                }
            };

            let path = paths.remove(expand_idx);
            let node = *path.last().unwrap();

            if node == target {
                #[cfg(debug_assertions)]
                unsafe {
                    ETHEREAL_PATH = Some(path.clone());
                }

                // calculate directions
                self.path = path
                    .iter()
                    .zip(path.iter().skip(1))
                    .map(|(a, b)| {
                        a.wrapping_dir_to_1(*b, board_dim)
                            .unwrap_or_else(|| panic!("no dir from {:?} to {:?}", a, b))
                    })
                    .collect();
                return;
            }

            for dir in Dir::iter() {
                let candidate = node.wrapping_translate(dir, 1, board_dim);
                if forbidden_positions.contains(&candidate) || seen.contains(&candidate) {
                    continue;
                }
                seen.insert(candidate);

                // inefficient path cloning
                let mut new_path = path.clone();
                new_path.push(candidate);
                paths.push(new_path);
            }
        }
    }

    // if there's no best path, at least avoid running into something
    fn least_damage(
        head: HexPoint,
        snake_body: &SnakeBody,
        other_snakes: OtherSnakes,
        board_dim: HexDim,
    ) -> Option<Dir> {
        let forbidden = snake_body
            .iter()
            .chain(other_snakes.iter_segments())
            .map(|seg| seg.pos)
            .collect::<HashSet<_>>();
        Dir::iter().find(|dir| !forbidden.contains(&head.wrapping_translate(*dir, 1, board_dim)))
    }
}

impl Controller for AStarAI {
    fn next_dir(
        &mut self,
        snake_body: &SnakeBody,
        other_snakes: OtherSnakes,
        apples: &[Apple],
        board_dim: HexDim,
    ) -> Option<Dir> {
        if let Some(p) = self.target {
            if p == snake_body[0].pos {
                // apple eaten
                self.target = None;
            }
        }

        if self.target.is_none()
            || self.path.is_empty()
            || self.steps_since_update >= Self::UPDATE_EVERY_N_STEPS
        {
            self.recalculate_target(snake_body[0].pos, apples);
            self.recalculate_path(snake_body, other_snakes, board_dim);
            self.steps_since_update = 0;
        }
        self.steps_since_update += 1;

        #[cfg(debug_assertions)]
        unsafe {
            match &mut ETHEREAL_PATH {
                Some(v) if !v.is_empty() => drop(v.remove(0)),
                _ => ETHEREAL_PATH = None,
            }
        }

        if !self.path.is_empty() {
            let dir = self.path.remove(0);
            Some(dir)
        } else {
            Self::least_damage(snake_body[0].pos, snake_body, other_snakes, board_dim)
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for AStarAI {
    fn drop(&mut self) {
        unsafe { ETHEREAL_PATH = None }
    }
}
