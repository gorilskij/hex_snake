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
    /// Shrunk down to the minimum length (hunger mode); frozen like `Crashed`
    Starved,
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

/// How far into its cell the head sinks before the black hole holds it there.
/// Half a cell puts the head tip at the cell's centre, where the hole is drawn.
const HOLE_DEPTH: f32 = 0.5;

/// Below this, a negative tail fraction is float noise rather than growth.
const TAIL_EPSILON: f32 = 1e-4;

/// A change to a snake's `length` spread over time. It eases out, so growth
/// starts out faster than the snake moves and the tail briefly backs up.
#[derive(Copy, Clone, Debug)]
pub struct LengthChange {
    amount: f32,
    duration: f32,
    elapsed: f32,
    applied: f32,
}

impl LengthChange {
    /// `amount` cells (negative to shrink) over `duration` seconds.
    pub fn new(amount: f32, duration: f32) -> Self {
        Self {
            amount,
            duration,
            elapsed: 0.,
            applied: 0.,
        }
    }

    /// Advance by `dt` seconds, returning how much length that adds.
    fn step(&mut self, dt: f32) -> f32 {
        self.elapsed += dt;
        let progress = if self.duration > 0. {
            (self.elapsed / self.duration).min(1.)
        } else {
            1.
        };
        let total = self.amount * ezing::cubic_out(progress);
        let delta = total - self.applied;
        self.applied = total;
        delta
    }

    fn is_done(&self) -> bool {
        self.elapsed >= self.duration
    }
}

pub struct Body {
    pub segments: VecDeque<Segment>,

    /// Direction the snake is currently going
    pub dir: Dir,

    /// The head's fractional progress into its leading cell (0..1). The head
    /// segment is `appearing` by this much.
    pub head_fraction: SegmentFraction,

    /// **The conserved quantity.** The snake's true length in cells, a float.
    /// A snake *is* this long from the moment it is created until it is gone;
    /// being born and dying do not change it, they only change how much of it
    /// is on the board (see [`Body::on_board`]). It changes only by digestion
    /// and [`LengthChange`]s in [`Snake::advance`] — never as a drifting difference of
    /// two accumulators.
    pub length: f32,

    /// How much material has come out of the birth hole, in cells. Chases
    /// `length` at exactly the head's speed, so until the snake is all the way
    /// out its tail stays pinned at the hole. Doubles as the regrowth animation
    /// after a cut: raise `length` and the difference emerges smoothly.
    pub emerged: f32,

    /// How much material has gone into the black hole, in cells. Grows at the
    /// head's speed once the head has sunk into the hole, until nothing is
    /// left. Never affects `length`.
    pub swallowed: f32,

    /// Changes to `length` still being applied.
    pub length_changes: Vec<LengthChange>,

    /// When a snake changes direction halfway through
    /// a segment appearing, the transition needs to be
    /// done smoothly, this indicates at which segment
    /// fraction the transition was started
    pub turn_start: Option<SegmentFraction>,

    /// For snakes that move using a search algorithm, this
    /// field remembers which cells were searched and which
    /// path is being followed, sored here to be drawn
    pub search_trace: Option<SearchTrace>,
}

impl Body {
    /// Add (or, if negative, remove) material right away. Once the snake is all
    /// the way out of its birth hole the change happens at the tail; before
    /// that, only the part still in the hole changes.
    fn change_length(&mut self, amount: f32) {
        let all_out = self.emerged >= self.length;
        self.length = (self.length + amount).max(0.);
        self.emerged = if all_out {
            self.length
        } else {
            self.emerged.min(self.length)
        };
    }

    /// The current length of the body (how many cells are visible)
    pub fn visible_len(&self) -> usize {
        self.segments.len()
    }

    /// How much of the snake is actually on the board, in cells: everything
    /// that has left the birth hole and not yet entered the black hole. The two
    /// holes are independent, so a snake can be emerging from one while
    /// disappearing into the other.
    pub fn on_board(&self) -> f32 {
        (self.emerged - self.swallowed).max(0.0)
    }

    /// How far the tail has receded out of its trailing cell (0..1). Purely
    /// derived, so it can never go stale; the last segment is `disappearing` by
    /// this much. Values `>= 1` mean the trailing segment is spent and should be
    /// popped — `advance` does that, so elsewhere the result is always in `0..1`.
    pub fn tail_fraction(&self) -> f32 {
        (self.visible_len() as f32 - 1.0) + self.head_fraction - self.on_board()
    }

    /// Digest what the tail passes over as it moves `travel` cells from
    /// `tail`, where it was (as a [`Self::tail_fraction`]), and return how
    /// much `length` grows by. Through an eaten segment with food left the
    /// tail moves at 1/(food+1) of `travel`'s pace, and the rest is growth,
    /// taken from the segment's `food_left`; anywhere else it moves at full
    /// pace. Growth slows the tail, so it may not get as far as `travel`.
    fn digest(&mut self, tail: f32, travel: f32) -> f32 {
        let mut growth = 0.;
        // the flow of material still to pass the tail this frame
        let mut flow = travel.max(0.);
        // how far into segment `i` the tail is
        let mut pos = tail.max(0.);
        let mut i = self.segments.len();

        while flow > 0. && i > 0 {
            i -= 1;
            let to_end = (1. - pos).max(0.);
            match &mut self.segments[i].segment_type {
                SegmentType::Eaten { original_food, food_left } if *food_left > 0. && *original_food > 0. => {
                    let food = *original_food;
                    // the flow that takes the tail to the end of the segment,
                    // and the flow that uses up the food left in it; they
                    // agree unless the segment was entered part-way
                    let flow_here = flow.min(to_end * (food + 1.)).min(*food_left * (food + 1.) / food);
                    let digested = (flow_here * food / (food + 1.)).min(*food_left);
                    *food_left -= digested;
                    growth += digested;
                    flow -= flow_here;
                    pos += flow_here / (food + 1.);
                    if *food_left > 0. && pos < 1. {
                        // the flow ran out inside the segment
                        break;
                    }
                    // whatever is left of the segment is crossed at full pace
                    let rest = flow.min((1. - pos).max(0.));
                    flow -= rest;
                    pos += rest;
                }
                _ => {
                    let moved = flow.min(to_end);
                    flow -= moved;
                    pos += moved;
                }
            }
            if pos < 1. {
                break;
            }
            pos -= 1.;
        }
        growth
    }
}

pub struct Snake {
    pub snake_type: Type,
    pub eat_mechanics: EatMechanics,
    /// Speed is measured in cells/s
    pub speed: f32,
    /// How fast the snake shrinks on its own, in cells/s (hunger mode)
    pub starvation: f32,

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

    /// The eaten segment the snake is already known to grow next, if any: its
    /// direction into the next cell is locked in and an `Eat` apple lies there.
    /// (Portals are not accounted for.)
    pub fn upcoming_eaten_segment(&self, apples: &[Apple], gtx: &GameContext) -> Option<SegmentType> {
        if self.state != State::Living || !self.dir_updated {
            return None;
        }
        let next_pos = self.head().pos.wrapping_translate(self.body.dir, 1, gtx.board_dim);
        apples
            .iter()
            .find(|apple| apple.pos == next_pos)
            .and_then(|apple| match apple.apple_type {
                crate::apple::Type::Eat(food) => Some(SegmentType::Eaten {
                    original_food: food,
                    food_left: food,
                }),
                _ => None,
            })
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
    pub fn advance(&mut self, elapsed: Duration, board_dim: HexDim) -> bool {
        if matches!(self.state, State::Crashed | State::Starved) {
            return false;
        }

        // Every frame, `delta` cells of material flow forward along the body.
        let delta = self.speed * elapsed.as_secs_f32();
        // where the tail starts this frame, for digestion
        let tail_start = self.body.tail_fraction();

        // Front end: either the head advances into the next cell, or — once it
        // has sunk into the death hole — it stays put and the material flowing
        // past it is swallowed instead.
        let mut cell_boundary_crossed = false;
        match self.state {
            State::Dying => {
                let sink = (HOLE_DEPTH - self.body.head_fraction).clamp(0.0, delta);
                self.body.head_fraction += sink;
                self.body.swallowed += delta - sink;
            }
            State::Living => {
                self.body.head_fraction += delta;
                if self.body.head_fraction >= 1.0 {
                    assert!(self.body.head_fraction < 2.0);
                    self.body.head_fraction -= 1.0;
                    self.dir_updated = false;
                    cell_boundary_crossed = true;
                }
            }
            State::Crashed | State::Starved => unreachable!("returned above"),
        }

        // Back end: material leaves the birth hole at the head's speed until
        // the whole snake is out. Must come before growth, so that growth
        // applied this frame is seen as "not yet emerged" and animates.
        self.body.emerged = (self.body.emerged + delta).min(self.body.length);

        // Length changes (apples, bad apples, starvation). Once the snake is all
        // the way out these move the tail directly, so growth faster than the
        // snake moves pushes the tail backwards.
        if self.state == State::Living {
            let dt = elapsed.as_secs_f32();
            let mut change = -self.starvation * dt;
            self.body.length_changes.retain_mut(|length_change| {
                change += length_change.step(dt);
                !length_change.is_done()
            });
            if change != 0. {
                self.body.change_length(change);
            }
        }

        // Digestion: while the tail is in an eaten segment it crosses it at
        // 1/(food+1) of the speed it would otherwise move, and the difference
        // is growth, so a segment grows `length` by exactly `food` once
        // crossed. The tail can cross segment boundaries mid-frame, so its
        // movement is followed segment by segment, each at its own rate. Only
        // a snake that is all the way out has a tail free to move at all. A
        // dying snake keeps digesting — the material is still flowing — which
        // does buy it a little time before the hole finishes it.
        let pending_push = cell_boundary_crossed as usize as f32;
        if self.body.emerged >= self.body.length {
            // how far the tail would move this frame without digestion
            let travel = self.body.tail_fraction() + pending_push - tail_start;
            let growth = self.body.digest(tail_start, travel);
            // like any length change of a snake that is all the way out, this
            // moves the tail directly, in this frame
            self.body.change_length(growth);
        }

        // Drop trailing segments the tail has fully receded past. A living
        // snake pushes one head segment this frame (in advance_cell) when it
        // crosses a boundary; count it now so the tail stays continuous across
        // that push.
        while self.body.tail_fraction() + pending_push >= 1.0 && self.body.visible_len() > 1 {
            self.body.segments.pop_back();
        }

        // The tail backing up past its last segment grows new segments behind
        // it, straight on from the last one.
        while self.body.tail_fraction() + pending_push < -TAIL_EPSILON {
            let last = *self.body.segments.back().expect("a snake always has a segment");
            let dir = last.coming_from;
            self.body.segments.push_back(Segment {
                segment_type: SegmentType::Normal,
                pos: last.pos.wrapping_translate(dir, 1, board_dim),
                coming_from: dir,
                going_to: Some(-dir),
                teleported: None,
                z_index: last.z_index,
            });
        }

        cell_boundary_crossed
    }

    pub fn advance_cell(&mut self, portals: &[Portal], gtx: &GameContext) {
        match &mut self.state {
            // a dying snake's head is pinned in the hole, so it never reaches a
            // cell boundary and advance() never reports one
            State::Dying => panic!("called advance_cell() on a dying snake"),
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
            State::Starved => panic!("called advance_cell() on a starved snake"),
        }

        self.body.turn_start = None;
        // Note: the tail is popped independently in advance(), not here — head
        // and tail cross their cell boundaries at different times.
    }

    /// Cut the snake starting from (and including) segment_index
    pub fn cut_at(&mut self, segment_index: usize) {
        let _ = self.body.segments.drain(segment_index..);

        // the cut leaves the tail flush with the start of the new last segment
        // (tail_fraction == 0); everything that is there is, by definition, out
        self.body.emerged = self.body.swallowed + (self.body.visible_len() as f32 - 1.0) + self.body.head_fraction;

        // ensure a length of at least 2 to avoid weird animation, otherwise
        // stop any previous growth. The shortfall regrows by emerging.
        self.body.length = self.body.emerged.max(2.0);
    }

    pub fn crash(&mut self) {
        if !matches!(self.state, State::Crashed) {
            self.state = State::Crashed;
            self.body.segments[0].segment_type = SegmentType::Crashed;
        }
    }

    pub fn starve(&mut self) {
        self.state = State::Starved;
    }

    pub fn die(&mut self) {
        if !matches!(self.state, State::Dying) {
            self.state = State::Dying;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::game_mode::GameMode;
    use crate::app::prefs::Prefs;
    use crate::apple::spawn::SpawnPolicy;
    use crate::basic::CellDim;
    use crate::snake::eat_mechanics::EatBehavior;

    const BOARD: HexDim = HexDim { h: 40, v: 40 };

    fn snake() -> Snake {
        builder::Builder::default()
            .snake_type(Type::Player)
            .eat_mechanics(EatMechanics::always(EatBehavior::PassOver))
            .palette(PaletteTemplate::rainbow())
            .controller(snake_control::Template::Programmed(vec![]))
            .speed(5.)
            .pos(HexPoint { h: 20, v: 20 })
            .dir(Dir::U)
            .len(4)
            .build()
            .unwrap()
    }

    fn step(snake: &mut Snake, dt: f32, gtx: &GameContext) {
        if snake.advance(Duration::from_secs_f32(dt), BOARD) {
            snake.advance_cell(&[], gtx);
        }
    }

    /// An apple grows the snake by exactly its food, whatever the frame
    /// length: the tail crosses segment boundaries mid-frame, and each part
    /// of its movement must be digested at its own segment's rate.
    #[test]
    fn an_apple_grows_the_snake_by_exactly_its_food() {
        let gtx = GameContext::new(
            BOARD,
            CellDim::from(10.),
            crate::app::Palette::dark(),
            Prefs::default(),
            SpawnPolicy::None,
            GameMode::Classic,
        );

        for dt in [1. / 60., 1. / 144., 1. / 7., 0.037, 0.0913] {
            for food in [1., 2., 0.5] {
                let mut snake = snake();
                // all the way out, and the head at an odd point in its cell
                for _ in 0..(10. / dt) as usize {
                    step(&mut snake, dt, &gtx);
                }
                let before = snake.body.length;

                // the head's cell becomes the eaten segment
                snake.body.segments[0].segment_type = SegmentType::Eaten { original_food: food, food_left: food };

                // until the tail has passed it
                for _ in 0..(20. / dt) as usize {
                    step(&mut snake, dt, &gtx);
                }
                let grown = snake.body.length - before;
                assert!((grown - food).abs() < 1e-4, "dt {dt}, food {food}: grew by {grown}");
            }
        }
    }
}
