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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SnakeState {
    Living,
    // counts how many segments have already been removed
    Dying(usize),
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
    Eaten { original_food: u32, food_left: u32 },
    Crashed,
    // does not advance, sucks the rest of the snake in
    BlackHole,
}

#[derive(Copy, Clone, Debug)]
pub struct Segment {
    pub typ: SegmentType,
    pub pos: HexPoint,
    // direction from this segment to the next one (towards the tail)
    pub next_segment: Dir,
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

// TODO: implement changing dir in the middle of a game frame
//  for graphical reasons
pub struct SnakeBody {
    pub cells: VecDeque<Segment>,
    pub dir: Dir,
    // prevent updating dir multiple times per game frame
    pub dir_grace: bool,
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
            next_segment: -dir,
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
                dir_grace: false,
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

    // TODO: this is still broken around edges, it doesn't show the cells within a
    //  manhattan distance if including teleportation, this is due to the way
    //  neighborhood is calculated, a correct calculation might be very involved
    pub fn head_neighborhood(&self, radius: usize, board_dim: HexDim) -> Vec<HexPoint> {
        self.head()
            .pos
            .neighborhood(radius)
            .into_iter()
            .filter_map(|point| point.wrap_around(board_dim, self.dir().axis()))
            .collect()
    }

    pub fn update_dir(&mut self, other_snakes: OtherSnakes, apples: &[Apple], board_dim: HexDim) {
        if !self.body.dir_grace && self.state == SnakeState::Living {
            if let Some(new_dir) =
                self.controller
                    .next_dir(&self.body, other_snakes, apples, board_dim)
            {
                self.body.dir = new_dir;
                self.body.dir_grace = true;
            }
        }
    }

    pub fn advance(&mut self, other_snakes: OtherSnakes, apples: &[Apple], board_dim: HexDim) {
        let last_idx = self.len() - 1;
        if let SegmentType::Eaten { food_left, .. } = &mut self.body.cells[last_idx].typ {
            if *food_left == 0 {
                self.body.cells[last_idx].typ = SegmentType::Normal;
            } else {
                self.body.grow += 1;
                *food_left -= 1;
            }
        }

        match &mut self.state {
            SnakeState::Dying(removed) => *removed += 1,
            SnakeState::Living => {
                self.update_dir(other_snakes, apples, board_dim);

                // create new head for snake
                let dir = self.dir();
                let new_head = Segment {
                    typ: SegmentType::Normal,
                    pos: self.head().pos.wrapping_translate(dir, 1, board_dim),
                    next_segment: -dir,
                    teleported: None,
                };
                self.body.cells.push_front(new_head);
            }
            SnakeState::Crashed => panic!("called advance() on a crashed snake"),
        }

        self.body.dir_grace = false;

        if self.body.grow > 0 {
            self.body.grow -= 1;
        } else {
            self.body.cells.pop_back();
        }
    }

    pub fn crash(&mut self) {
        if !matches!(self.state, SnakeState::Crashed) {
            self.state = SnakeState::Crashed;
            self.body.cells[0].typ = SegmentType::Crashed;
        }
    }

    pub fn die(&mut self) {
        if !matches!(self.state, SnakeState::Dying(_)) {
            self.state = SnakeState::Dying(0);
            self.body.cells[0].typ = SegmentType::BlackHole;
        }
    }
}
