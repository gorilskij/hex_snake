use std::collections::{HashMap, VecDeque, HashSet};

use crate::{
    app::{
        game::Apple,
        snake::{
            controller::{Controller, ControllerTemplate, OtherSnakes},
            palette::{Palette, PaletteTemplate},
        },
        Frames,
    },
    basic::{Dir, HexDim, HexPoint},
};
use std::ops::Deref;
use crate::app::game::FrameStamp;

pub mod controller;
pub mod palette;
pub mod rendering;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SnakeState {
    Living,
    Dying,
    Crashed,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum SnakeType {
    Player,
    Simulated {
        start_pos: HexPoint,
        start_dir: Dir,
        start_grow: usize,
    },
    Competitor {
        life: Option<Frames>,
    },
    Killer {
        life: Option<Frames>,
    },
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
    /// Direction from this segment to the next one (towards the tail)
    pub coming_from: Dir,
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

pub struct SearchTrace {
    pub cells_searched: HashSet<HexPoint>,
    pub current_path: Vec<HexPoint>,
}

pub struct SnakeBody {
    pub cells: VecDeque<Segment>,

    /// When a snake is being destroyed from the front
    /// (when it's falling into a black hole), this is
    /// used to indicate how many segments are missing
    /// off the front (how many are in the black hole)
    pub missing_front: usize,

    /// Direction the snake is currently going
    pub dir: Dir,

    /// When a snake changes direction halfway through
    /// a segment appearing, the transition needs to be
    /// done smoothly, this indicates at which frame and
    /// frame_frac the transition was started
    pub turn_start: Option<(usize, f32)>,

    /// When `Snake::update_dir` is called from a draw method
    /// (this is done to show the snake turning as soon
    /// as possible), dir_grace prevents a repeat call
    /// arising from a subsequent call to `Snake::advance`
    pub dir_grace: bool,
    pub grow: usize,
    /// For snakes that move using a search algorithm, this
    /// field remembers which cells were searched and which
    /// path is being followed, sored here to be drawn
    pub search_trace: Option<SearchTrace>,
}

impl Deref for SnakeBody {
    type Target = VecDeque<Segment>;

    fn deref(&self) -> &Self::Target {
        &self.cells
    }
}

pub struct Snake {
    pub snake_type: SnakeType,
    pub eat_mechanics: EatMechanics,

    pub body: SnakeBody,
    pub state: SnakeState,

    pub controller: Box<dyn Controller>,
    pub palette: Box<dyn Palette>,
}

#[derive(Clone)]
pub struct SnakeSeed {
    pub snake_type: SnakeType,
    pub eat_mechanics: EatMechanics,
    pub palette: PaletteTemplate,
    pub controller: ControllerTemplate,
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
            coming_from: -dir,
            teleported: None,
        };

        let mut body = VecDeque::new();
        body.push_back(head);

        Self {
            snake_type,
            eat_mechanics,

            body: SnakeBody {
                cells: body,
                missing_front: 0,
                dir,
                turn_start: None,
                dir_grace: false,
                grow,
                search_trace: None,
            },
            state: SnakeState::Living,

            controller: controller.into_controller(dir),
            palette: palette.into(),
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

    // similar to reachable(..), much more efficient, only works in the plane,
    // doesn't account for the snake itself
    // pub fn head_neighborhood(&self, radius: usize, board_dim: HexDim) -> Vec<HexPoint> {
    //     self.head()
    //         .pos
    //         .neighborhood(radius)
    //         .into_iter()
    //         .filter_map(|point| point.wrap_around(board_dim, self.dir().axis()))
    //         .collect()
    // }

    // very inefficient
    // all points theoretically reachable in 'radius' steps (assumes no cutting)
    pub fn reachable(&self, radius: usize, board_dim: HexDim) -> Vec<HexPoint> {
        let mut out = vec![];
        let mut layer = vec![self.head().pos];

        // excluding the point itself
        fn immediate_neighborhood(point: HexPoint, board_dim: HexDim) -> Vec<HexPoint> {
            // could exclude -(current dir) but that might not be worth it overall
            Dir::iter()
                .map(|dir| point.wrapping_translate(dir, 1, board_dim))
                .collect()
        }

        fn snake_contains(snake: &Snake, point: HexPoint) -> bool {
            snake.body.cells.iter().any(|segment| segment.pos == point)
        }

        for _ in 0..radius {
            let mut new: Vec<_> = layer
                .iter()
                .flat_map(|point| immediate_neighborhood(*point, board_dim).into_iter())
                .collect();
            new.sort_unstable();
            new.dedup();
            new.retain(|x| !out.contains(x) && !snake_contains(self, *x));
            out.extend_from_slice(&new);
            layer = new;
        }

        out
    }

    pub fn update_dir(&mut self, other_snakes: OtherSnakes, apples: &[Apple], board_dim: HexDim, frame_stamp: FrameStamp) {
        if !self.body.dir_grace && self.state == SnakeState::Living {
            if let Some(new_dir) =
                self.controller
                    .next_dir(&mut self.body, other_snakes, apples, board_dim)
            {
                self.body.dir = new_dir;
                self.body.dir_grace = true;
                self.body.turn_start = Some(frame_stamp);
            }
        }
    }

    pub fn advance(&mut self, other_snakes: OtherSnakes, apples: &[Apple], board_dim: HexDim, frame_stamp: FrameStamp) {
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
            SnakeState::Dying => self.body.missing_front += 1,
            SnakeState::Living => {
                self.update_dir(other_snakes, apples, board_dim, frame_stamp);

                // create new head for snake
                let dir = self.dir();
                let new_head = Segment {
                    typ: SegmentType::Normal,
                    // this gets very interesting if you move 2 cells each time
                    // (porous snake)
                    pos: self.head().pos.wrapping_translate(dir, 1, board_dim),
                    coming_from: -dir,
                    teleported: None,
                };
                self.body.cells.push_front(new_head);
            }
            SnakeState::Crashed => panic!("called advance() on a crashed snake"),
        }

        self.body.dir_grace = false;
        self.body.turn_start = None;

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
        if !matches!(self.state, SnakeState::Dying) {
            self.state = SnakeState::Dying;
            self.body.cells[0].typ = SegmentType::BlackHole;
        }
    }
}
