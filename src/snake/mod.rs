use std::collections::{HashSet, VecDeque};
use std::time::Duration;

use enum_map_lite::Enum;
pub use palette::{Palette, PaletteTemplate};

use crate::app::game_context::GameContext;
use crate::app::portal::{Behavior, Portal};
use crate::apple::Apple;
use crate::basic::{Dir, Frames, HexDim, HexPoint};
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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Enum)]
pub enum Type {
    Player,
    Simulated,
    Competitor { life: Option<Frames> },
    Killer { life: Option<Frames> },
    Rain,
}

#[derive(PartialEq, Copy, Clone, Debug, Enum)]
pub enum SegmentType {
    Normal,
    // original_food sets the digestion rate (1/(food+1)); food_left is the
    // growth still to be delivered, so total growth is capped at exactly food
    Eaten { original_food: f32, food_left: f32 },
    Crashed,
    // does not advance, sucks the rest of the snake in
    BlackHole { just_created: bool },
}

pub type ZIndex = i32;

#[derive(Copy, Clone, Debug)]
pub struct Segment {
    pub segment_type: SegmentType,
    pub pos: HexPoint,
    /// Direction from this segment to the next one (towards the tail)
    pub coming_from: Dir,
    // going_to should be set if and only if the segment is not the head
    pub going_to: Option<Dir>,
    pub teleported: Option<Dir>,
    pub z_index: ZIndex,
}

pub struct SearchTrace {
    pub cells_searched: HashSet<HexPoint>,
    pub current_path: Vec<HexPoint>,
}

type SegmentFraction = f32;

pub struct Body {
    pub segments: VecDeque<Segment>,

    /// When a snake is being destroyed from the front
    /// (when it's falling into a black hole), this is
    /// used to indicate how many segments are missing
    /// off the front (how many are in the black hole)
    pub missing_front: usize,

    /// Direction the snake is currently going
    pub dir: Dir,

    /// The head's fractional progress into its leading cell (0..1). The head
    /// segment is `appearing` by this much.
    pub head_fraction: SegmentFraction,

    /// **The conserved quantity.** The snake's true length in cells, a float.
    /// It only ever changes by explicit, capped growth in [`Snake::advance`] —
    /// never as a drifting difference of two accumulators. The tail position
    /// ([`Body::tail_fraction`], and which segments exist) is *derived* from it:
    /// `length == (visible_len - 1) + head_fraction - tail_fraction`.
    pub length: f32,

    /// When a snake changes direction halfway through
    /// a segment appearing, the transition needs to be
    /// done smoothly, this indicates at which segment
    /// fraction the transition was started
    pub turn_start: Option<SegmentFraction>,

    /// Pending whole-cell growth from a source other than digestion (initial
    /// length, cut regrowth), in cells. While positive it freezes the tail
    /// (length grows at the head's rate) until paid off. Digestion grows
    /// `length` directly and does not use this.
    pub grow: f32,
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

    /// How far the tail has receded out of its trailing cell (0..1). Purely
    /// derived from [`Self::length`] (the conserved quantity), so it can never
    /// go stale; the last segment is `disappearing` by this much.
    ///
    /// Values `>= 1` mean the trailing segment is spent and should be popped —
    /// `advance` does that, so outside of it the result is always in `0..1`.
    /// Values `< 0` mean `length` exceeds the material actually on the board,
    /// which happens while `grow` is pending; the tail is drawn solid then, so
    /// the value is unused.
    pub fn tail_fraction(&self) -> f32 {
        (self.visible_len() as f32 - 1.0) + self.head_fraction - self.length
    }
}

pub struct Snake {
    pub snake_type: Type,
    pub eat_mechanics: EatMechanics,
    /// Speed is measured in cells/s
    pub speed: f32,

    pub body: Body,
    pub state: State,
    /// Indicates whether the controller already set a new direction
    /// within this cell. Direction changes can happen at most once
    /// per cell.
    pub dir_updated: bool,

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

    pub fn update_dir(&mut self, other_snakes: impl Snakes, apples: &[Apple], gtx: &GameContext) {
        if self.state != State::Living {
            // to avoid calling this function again
            self.dir_updated = true;
            return;
        }

        // advance controller
        let knowledge = Knowledge::accurate(&self.eat_mechanics);
        let controller_dir = self
            .controller
            .next_dir(&mut self.body, Some(&knowledge), &other_snakes, apples, gtx);

        let new_dir = if self.autopilot_control {
            self.autopilot
                .as_mut()
                .map(|autopilot| autopilot.next_dir(&mut self.body, Some(&knowledge), &other_snakes, apples, gtx))
                .expect("autopilot_control == true with missing autopilot")
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
            Some(dir) => {
                self.dir_updated = true;
                self.body.dir = dir;
                self.body.turn_start = Some(self.body.head_fraction);
            }
            _ => {}
        }
    }

    /// Return value indicates whether a call to advance_cell should be made
    pub fn advance(&mut self, elapsed: Duration) -> bool {
        if self.state == State::Crashed {
            return false;
        }

        let delta = self.speed * elapsed.as_secs_f32();

        // Head: advance into the leading cell; a new head segment is pushed at
        // the boundary (in advance_cell for a living snake).
        self.body.head_fraction += delta;
        let mut cell_boundary_crossed = false;
        if self.body.head_fraction >= 1.0 {
            // TODO: might need to do multiple calls to advance_cell at high speeds
            assert!(self.body.head_fraction < 2.0);
            self.body.head_fraction -= 1.0;
            self.dir_updated = false;
            cell_boundary_crossed = true;
        }

        // Grow `length` (the conserved quantity) by explicit, capped amounts.
        // The head moves at full speed; all growth is the tail *lagging*, i.e.
        // length increasing. Nothing here subtracts accumulators, so length
        // cannot drift.
        if self.body.grow > 0.0 {
            // Pending whole-cell growth (initial length, cut regrowth): the
            // tail is frozen — length grows at the head's rate until paid off.
            let step = delta.min(self.body.grow);
            self.body.length += step;
            self.body.grow -= step;
        } else if self.state == State::Living {
            // Digestion: while the tail segment is eaten, the tail crosses it at
            // 1/(food+1) speed, so length grows at food/(food+1) of the head's
            // rate — and lags by exactly `food` once fully crossed. Capped at
            // `food_left` so the total is exactly `food`, drift-free.
            if let Some(SegmentType::Eaten { original_food, food_left }) =
                self.body.segments.back().map(|s| s.segment_type)
            {
                if food_left > 0.0 {
                    let step = (delta * original_food / (original_food + 1.0)).min(food_left);
                    self.body.length += step;
                    if let Some(SegmentType::Eaten { food_left, .. }) =
                        self.body.segments.back_mut().map(|s| &mut s.segment_type)
                    {
                        *food_left -= step;
                    }
                }
            }
        }

        // Derive the tail from `length` (never accumulated):
        //   length == (visible_len - 1) + head_fraction - tail_fraction
        // Living snakes pop from the tail here; dying/crashed snakes keep their
        // discrete black-hole shrink in advance_cell, so leave them alone.
        if self.state == State::Living {
            // A living snake pushes one head segment this frame (in advance_cell)
            // when it crosses a boundary; count it now so the tail stays
            // continuous across that push.
            let pending_push = cell_boundary_crossed as usize as f32;
            while self.body.tail_fraction() + pending_push >= 1.0 && self.body.visible_len() > 1 {
                self.body.segments.pop_back();
            }
        }

        cell_boundary_crossed
    }

    pub fn advance_cell(&mut self, portals: &[Portal], gtx: &GameContext) {
        match &mut self.state {
            State::Dying => {
                // a dying snake shrinks from the front (sucked into the black
                // hole) and from the tail (discrete pop per cell — the derived
                // tail in advance() is for living snakes only). Keep `length` in
                // step with the popped tail so the gradient stays sensible.
                self.body.missing_front += 1;
                if self.body.segments.pop_back().is_some() {
                    self.body.length = (self.body.length - 1.0).max(0.0);
                }
            }
            State::Living => {
                // create new head for snake
                let dir = self.body.dir;

                let head_pos = self.head().pos;
                let new_head_pos_raw = head_pos.translate(dir, 1);
                let mut new_head_pos = head_pos.wrapping_translate(dir, 1, gtx.board_dim);

                let mut dir_changed_in_teleport = None;
                for portal in portals {
                    match portal.check(head_pos, new_head_pos_raw) {
                        Some(Behavior::Die) => self.die(),
                        Some(Behavior::TeleportTo(dest, new_dir)) => {
                            new_head_pos = dest;
                            if dir != new_dir {
                                dir_changed_in_teleport = Some(new_dir);
                            }
                            self.body.dir = new_dir;
                        }
                        Some(Behavior::WrapAround) => {
                            println!("TODO: implement")
                        }
                        Some(Behavior::PassThrough) | Some(Behavior::Nothing) | None => {}
                        Some(Behavior::Unreachable) => panic!("Tried to execute unreachable portal behavior"),
                    }
                }

                let new_dir = dir_changed_in_teleport.unwrap_or(dir);
                let new_head = Segment {
                    segment_type: SegmentType::Normal,
                    // this gets very interesting if you move 2 cells each time
                    // (porous snake)
                    pos: new_head_pos,
                    coming_from: -new_dir,
                    going_to: None,
                    teleported: None,
                    z_index: 0,
                };
                self.body.segments[0].going_to = Some(dir);
                self.body.segments.push_front(new_head);
            }
            State::Crashed => panic!("called advance_cell() on a crashed snake"),
        }

        self.body.turn_start = None;
        // Note: the tail is popped independently in advance(), not here — head
        // and tail cross their cell boundaries at different times.
    }

    /// Cut the snake starting from (and including) segment_index
    pub fn cut_at(&mut self, segment_index: usize) {
        let _ = self.body.segments.drain(segment_index..);

        // reset length to the freshly-cut body (tail at the start of the new
        // last segment, i.e. tail_fraction == 0), keeping length the source of truth
        self.body.length = (self.body.visible_len() as f32 - 1.0) + self.body.head_fraction;

        // ensure a length of at least 2 to avoid weird animation,
        // otherwise, stop any previous growth
        self.body.grow = (2.0 - self.body.visible_len() as f32).max(0.0);
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
