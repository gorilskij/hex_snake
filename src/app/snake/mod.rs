use std::collections::{HashMap, VecDeque};

use crate::app::{
    game::Apple,
    hex::{Dir, HexDim, HexPoint},
    snake::{
        controller::{OtherSnakes, SnakeController, SnakeControllerTemplate},
        palette::{SnakePainter, SnakePaletteTemplate},
    },
    Frames,
};

pub mod controller;
pub mod palette;

#[derive(Eq, PartialEq)]
pub enum SnakeState {
    Living,
    Dying,
    Crashed,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum SnakeType {
    PlayerSnake,
    SimulatedSnake,
    CompetitorSnake { life: Option<Frames> },
    KillerSnake { life: Option<Frames> },
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum SegmentType {
    Normal,
    Eaten(u32),
    Crashed,
    BlackHole, // does not advance, sucks the rest of the snake in
}

#[derive(Copy, Clone, Debug)]
pub struct Segment {
    pub typ: SegmentType,
    pub pos: HexPoint,
    pub next_segment: Option<Dir>,
    pub teleported: Option<Dir>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum EatBehavior {
    Cut,   // cut the other snake's tail off
    Crash, // stop the game
    Die,   // disappear
}

#[derive(Clone)]
pub struct EatMechanics {
    pub eat_self: EatBehavior,
    pub eat_other: HashMap<SnakeType, EatBehavior>,
    pub default: EatBehavior,
}

impl EatMechanics {
    pub fn always(behavior: EatBehavior) -> Self {
        Self {
            eat_self: behavior,
            eat_other: hash_map! {},
            default: behavior,
        }
    }
}

pub struct SnakeBody {
    pub cells: VecDeque<Segment>,
    pub dir: Dir,
    pub grow: usize,
}

pub struct Snake {
    pub snake_type: SnakeType,
    pub eat_mechanics: EatMechanics,

    pub body: SnakeBody,
    pub state: SnakeState,

    pub controller: Box<dyn SnakeController>,
    pub painter: Box<dyn SnakePainter>,
}

#[derive(Clone)]
pub struct SnakeSeed {
    pub snake_type: SnakeType,
    pub eat_mechanics: EatMechanics,
    pub palette: SnakePaletteTemplate,
    pub controller: SnakeControllerTemplate,
}

impl Snake {
    pub fn from_seed(seed: &SnakeSeed, pos: HexPoint, dir: Dir, grow: usize) -> Self {
        let SnakeSeed {
            snake_type,
            eat_mechanics,
            palette,
            controller,
        } = (*seed).clone();

        let head = Segment {
            typ: SegmentType::Normal,
            pos,
            next_segment: None,
            teleported: None,
        };

        let mut body = VecDeque::new();
        body.push_back(head);

        Self {
            snake_type,
            eat_mechanics,

            body: SnakeBody {
                cells: body,
                dir,
                grow,
            },
            state: SnakeState::Living,

            controller: controller.into_controller(dir),
            painter: palette.into(),
        }
    }

    pub fn len(&self) -> usize {
        self.body.cells.len()
    }

    pub fn dir(&self) -> Dir {
        self.body.dir
    }

    pub fn head(&self) -> &Segment {
        &self.body.cells[0]
    }

    pub fn advance(&mut self, other_snakes: OtherSnakes, apples: &[Apple], board_dim: HexDim) {
        let last_idx = self.len() - 1;
        if let SegmentType::Eaten(amount) = &mut self.body.cells[last_idx].typ {
            if *amount == 0 {
                self.body.cells[last_idx].typ = SegmentType::Normal;
            } else {
                self.body.grow += 1;
                *amount -= 1;
            }
        }

        let dir;
        if self.state != SnakeState::Dying {
            // determine new direction for snake
            if let Some(new_dir) =
                self.controller
                    .next_dir(&self.body, other_snakes, apples, board_dim)
            {
                self.body.dir = new_dir;
                dir = new_dir;
            } else {
                dir = self.dir();
            }

            // create new head for snake
            let new_head = Segment {
                typ: SegmentType::Normal,
                pos: self.head().pos.wrapping_translate(dir, 1, board_dim),
                next_segment: None,
                teleported: None,
            };
            self.body.cells.push_front(new_head);
            self.body
                .cells
                .get_mut(1)
                .map(|neck| neck.next_segment = Some(dir));
        }

        if self.body.grow > 0 {
            self.body.grow -= 1;
        } else {
            self.body.cells.pop_back();
        }
    }
}
