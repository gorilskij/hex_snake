use std::collections::{HashSet, VecDeque};
use std::mem;
use std::mem::Discriminant;

use ggez::Context;
pub use palette::{Palette, PaletteTemplate};

use crate::app::fps_control::FpsContext;
use crate::app::game_context::GameContext;
use crate::app::portal::{Behavior, Portal};
use crate::apple::Apple;
use crate::basic::{Dir, FrameStamp, Frames, HexDim, HexPoint};
use crate::snake::eat_mechanics::{EatMechanics, Knowledge};
use crate::snake_control;
use crate::snake_control::{pathfinder, Controller};
use crate::view::snakes::Snakes;

pub mod builder;
pub mod eat_mechanics;
pub mod palette;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum State {
    Living,
    Dying,
    Crashed,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Type {
    Player,
    Simulated,
    Competitor { life: Option<Frames> },
    Killer { life: Option<Frames> },
    Rain,
}

// NOTE: if variants are added, the code should be checked for
//       usages of Discriminant<SegmentType>, match statements
//       using this type should be extended accordingly
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum SegmentType {
    Normal,
    Eaten { original_food: u32, food_left: u32 },
    Crashed,
    // does not advance, sucks the rest of the snake in
    BlackHole { just_created: bool },
}

impl SegmentType {
    pub fn discriminant(&self) -> Discriminant<Self> {
        mem::discriminant(self)
    }

    pub const DISCR_NORMAL: Discriminant<Self> = mem::discriminant(&Self::Normal);
    pub const DISCR_EATEN: Discriminant<Self> = mem::discriminant(&Self::Eaten { original_food: 0, food_left: 0 });
    pub const DISCR_CRASHED: Discriminant<Self> = mem::discriminant(&Self::Crashed);
    pub const DISCR_BLACK_HOLE: Discriminant<Self> = mem::discriminant(&Self::BlackHole { just_created: false });
}

pub type ZIndex = i32;

#[derive(Copy, Clone, Debug)]
pub struct Segment {
    pub segment_type: SegmentType,
    pub pos: HexPoint,
    /// Direction from this segment to the next one (towards the tail)
    pub coming_from: Dir,
    pub teleported: Option<Dir>,
    pub z_index: ZIndex,
}

pub struct SearchTrace {
    pub cells_searched: HashSet<HexPoint>,
    pub current_path: Vec<HexPoint>,
}

pub struct Body {
    pub segments: VecDeque<Segment>,

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
    /// frame fraction the transition was started
    pub turn_start: Option<FrameStamp>,

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

impl Body {
    /// The current length of the body (how many cells are visible)
    pub fn visible_len(&self) -> usize {
        self.segments.len()
    }

    /// The full logical length of the snake, including the part that
    /// is inside a black hole when the snake is dying
    pub fn logical_len(&self) -> usize {
        self.segments.len() + self.missing_front
    }
}

pub struct Snake {
    pub snake_type: Type,
    pub eat_mechanics: EatMechanics,
    pub speed: f32,

    pub body: Body,
    pub state: State,

    pub controller: Box<dyn Controller + Send + Sync>,
    pub palette: Box<dyn Palette + Send + Sync>,

    pub autopilot: Option<Box<dyn Controller + Send + Sync>>,
    pub autopilot_control: bool, // whether autopilot is in control
}

impl Snake {
    pub fn head(&self) -> &Segment {
        &self.body.segments[0]
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
            snake.body.segments.iter().any(|segment| segment.pos == point)
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

    pub fn update_dir(
        &mut self,
        other_snakes: impl Snakes,
        apples: &[Apple],
        gtx: &GameContext,
        ftx: &FpsContext,
        ctx: &Context,
    ) {
        if self.body.dir_grace || self.state != State::Living {
            return;
        }

        // advance controller
        let knowledge = Knowledge::accurate(&self.eat_mechanics);
        let controller_dir =
            self.controller
                .next_dir(&mut self.body, Some(&knowledge), &other_snakes, apples, gtx, ftx, ctx);

        // advance autopilot
        let autopilot_dir = self.autopilot.as_mut().map(|autopilot| {
            autopilot.next_dir(&mut self.body, Some(&knowledge), &other_snakes, apples, gtx, ftx, ctx)
        });

        let new_dir = if self.autopilot_control {
            autopilot_dir.expect("autopilot_control == true with missing autopilot")
        } else {
            controller_dir
        };

        match new_dir {
            Some(dir) if dir == -self.body.dir => {
                eprintln!(
                    "warning: snake_control tried to perform a 180° turn {:?} -> {:?}",
                    self.body.dir, dir
                );
            }
            Some(dir) if dir == self.body.dir => {
                // if the controller returns Some with the same direction,
                // this does not lead to a turn but it does prevent the snake
                // from calling next_dir until the next cell
                self.body.dir_grace = true;
            }
            Some(dir) => {
                self.body.dir = dir;
                self.body.dir_grace = true;
                self.body.turn_start = Some(ftx.last_graphics_update);
            }
            _ => {}
        }
    }

    pub fn advance(
        &mut self,
        other_snakes: impl Snakes,
        apples: &[Apple],
        portals: &[Portal],
        gtx: &GameContext,
        ftx: &FpsContext,
        ctx: &Context,
    ) {
        let last_idx = self.body.visible_len() - 1;
        if let SegmentType::Eaten { food_left, .. } = &mut self.body.segments[last_idx].segment_type {
            if *food_left == 0 {
                self.body.segments[last_idx].segment_type = SegmentType::Normal;
            } else {
                self.body.grow += 1;
                *food_left -= 1;
            }
        }

        match &mut self.state {
            State::Dying => self.body.missing_front += 1,
            State::Living => {
                self.update_dir(other_snakes, apples, gtx, ftx, ctx);

                // create new head for snake
                let dir = self.body.dir;

                let head_pos = self.head().pos;
                let mut new_head_pos = head_pos.wrapping_translate(dir, 1, gtx.board_dim);

                for portal in portals {
                    match portal.check(head_pos, new_head_pos) {
                        Some(Behavior::Die) => self.die(),
                        Some(Behavior::Teleport) => {
                            println!("TODO: implement")
                        }
                        Some(Behavior::WrapAround) => {
                            println!("TODO: implement")
                        }
                        Some(Behavior::PassThrough) | None => {}
                    }
                }

                let new_head = Segment {
                    segment_type: SegmentType::Normal,
                    // this gets very interesting if you move 2 cells each time
                    // (porous snake)
                    pos: new_head_pos,
                    coming_from: -dir,
                    teleported: None,
                    z_index: 0,
                };
                self.body.segments.push_front(new_head);
            }
            State::Crashed => panic!("called advance() on a crashed snake"),
        }

        self.body.dir_grace = false;
        self.body.turn_start = None;

        if self.body.grow > 0 {
            self.body.grow -= 1;
        } else {
            self.body.segments.pop_back();
        }
    }

    /// Cut the snake starting from (and including) segment_index
    pub fn cut_at(&mut self, segment_index: usize) {
        let _ = self.body.segments.drain(segment_index..);

        // ensure a length of at least 2 to avoid weird animation,
        // otherwise, stop any previous growth
        self.body.grow = 2_usize.saturating_sub(self.body.visible_len());
    }

    pub fn crash(&mut self) {
        if !matches!(self.state, State::Crashed) {
            self.state = State::Crashed;
            self.body.segments[0].segment_type = SegmentType::Crashed;
        }
    }

    pub fn die(&mut self) {
        if !matches!(self.state, State::Dying) {
            self.state = State::Dying;
            self.body.segments[0].segment_type = SegmentType::BlackHole { just_created: true };
        }
    }
}
