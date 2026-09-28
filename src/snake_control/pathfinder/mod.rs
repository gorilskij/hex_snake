mod space_filling;
mod weighted_bfs;
mod with_backup;

use std::collections::{HashMap, HashSet, VecDeque};

use space_filling::SpaceFilling;
use weighted_bfs::WeightedBFS;
pub use weighted_bfs::Weights;
use with_backup::WithBackup;

use crate::app::game_context::GameContext;
use crate::basic::{Dir, HexDim, HexPoint};
use crate::snake::Body;
use crate::snake::eat_mechanics::Knowledge;
use crate::view::snakes::Snakes;

pub type Path = VecDeque<HexPoint>;

/// Where a search starts: a cell and the heading arrived with, since turn costs
/// make the two inseparable. Usually the head, but a leg that continues an
/// earlier one starts where that one ended.
pub type Start = (HexPoint, Dir);

/// One search's worth of path: the cells, and the target it ends on.
///
/// `target` is `None` for a path that isn't heading anywhere in particular (the
/// survival crawl), which is also what tells a follower not to get attached
/// to it.
#[derive(Clone, Debug)]
pub struct Leg {
    pub cells: Path,
    pub target: Option<HexPoint>,
}

impl Leg {
    /// The heading a snake arrives at the end with — where a continuing leg
    /// has to start from.
    ///
    /// Not `dir_to`: across the board's edge that points the opposite way.
    pub fn arrival(&self, board_dim: HexDim) -> Option<Start> {
        let [before, last] = [self.cells.len().checked_sub(2)?, self.cells.len() - 1];
        let (before, last) = (self.cells[before], self.cells[last]);
        Some((last, before.single_step_dir_to(last, board_dim)?))
    }
}

/// A route through several targets, one [`Leg`] per target, each starting where
/// the previous one ended.
///
/// This is what a follower keeps and a renderer draws; building it out of legs
/// is path management, deliberately kept out of the searches themselves.
#[derive(Clone, Debug)]
pub struct Plan {
    pub legs: Vec<Leg>,
    /// The board it was planned on: steps across the edge only connect on a
    /// board of that size.
    pub board_dim: HexDim,
}

impl Plan {
    /// Whether any leg is going nowhere in particular, in which case the plan
    /// is worth reconsidering as soon as possible.
    pub fn is_fallback(&self) -> bool {
        self.legs.iter().any(|leg| leg.target.is_none())
    }

    /// The targets still to be reached, in order.
    pub fn targets(&self) -> impl Iterator<Item = HexPoint> + '_ {
        self.legs.iter().filter_map(|leg| leg.target)
    }

    /// Every cell of the route, head first. Legs share the cell they meet in,
    /// which is what makes them one continuous route; this yields it once.
    pub fn cells(&self) -> impl Iterator<Item = HexPoint> + '_ {
        self.legs.iter().enumerate().flat_map(|(idx, leg)| {
            let shared = if idx == 0 { 0 } else { 1 };
            leg.cells.iter().skip(shared).copied()
        })
    }
}

/// What the plan this leg extends has already committed to.
///
/// A route is a promise about where the snake will be, so the cells it already
/// uses are in the way of anything planned after it — that is what stops a
/// later leg from crossing an earlier one.
#[derive(Copy, Clone, Default)]
pub struct Committed<'a> {
    /// Cells the route already uses, and what the snake expects to find there
    /// by the time it comes back around: its own body, or (at a target it will
    /// have eaten) a segment it may be able to pass through.
    pub cells: &'a [(HexPoint, Obstacle)],
    /// Targets earlier legs are already going for, which are therefore no
    /// longer targets for this one.
    pub targets: &'a [HexPoint],
}

/// What a search is after (see [`Appetite`]).
///
/// [`Appetite`]: crate::snake_control::appetite::Appetite
#[derive(Clone, Debug, Default)]
pub struct Goals {
    /// Cells worth getting to.
    pub targets: Vec<HexPoint>,
    /// Cells worth going around, and how much extra crossing each one costs.
    pub avoid: HashMap<HexPoint, u32>,
}

/// A search for one leg: the cheapest way from `start` to any target that isn't
/// already taken, going around what the plan has already `committed` to.
///
/// Collecting several targets is not a mode of the search — it is the caller
/// chaining legs, each starting where the last one ended (see [`Plan`]).
pub trait PathFinder {
    #[allow(clippy::too_many_arguments)]
    fn get_path(
        &self,
        start: Start,
        goals: &Goals,
        committed: Committed,
        body: &Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        gtx: &GameContext,
    ) -> Option<Leg>;
}

#[derive(Clone, Debug)]
pub enum Template {
    WeightedBFS(Weights),
    SpaceFilling,
    WithBackup { main: Box<Template>, backup: Box<Template> },
}

impl Template {
    pub fn into_pathfinder(self) -> Box<dyn PathFinder + Send + Sync> {
        match self {
            Template::WeightedBFS(weights) => Box::new(WeightedBFS { weights }),
            Template::SpaceFilling => Box::new(SpaceFilling),
            Template::WithBackup { main, backup } => Box::new(WithBackup {
                main: main.into_pathfinder(),
                backup: backup.into_pathfinder(),
            }),
        }
    }
}

/// Every cell reachable from `start` without running into anything (`start`
/// included), and the blocked cells walling them in.
pub fn surroundings(
    start: HexPoint,
    board_dim: HexDim,
    blocks: impl Fn(HexPoint) -> bool,
) -> (HashSet<HexPoint>, HashSet<HexPoint>) {
    let mut region = HashSet::from([start]);
    let mut walls = HashSet::new();
    let mut stack = vec![start];
    while let Some(pos) = stack.pop() {
        for dir in Dir::iter() {
            let next = pos.wrapping_translate(dir, 1, board_dim);
            if blocks(next) {
                walls.insert(next);
            } else if region.insert(next) {
                stack.push(next);
            }
        }
    }
    (region, walls)
}

/// What a segment means to a head entering its cell, in increasing order of
/// severity.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Obstacle {
    Passable,
    Blocked,
}

/// Every occupied cell as seen by one snake. Without knowledge, every segment
/// blocks.
pub struct Obstacles(HashMap<HexPoint, Obstacle>);

impl Obstacles {
    pub fn new(body: &Body, knowledge: Option<&Knowledge>, other_snakes: &dyn Snakes) -> Self {
        let own = body
            .segments
            .iter()
            .map(|seg| (seg.pos, knowledge.is_some_and(|k| k.can_pass_through_self(seg))));
        // other snakes are judged by how this snake eats *them*, not itself
        let others = other_snakes.iter().flat_map(|snake| {
            snake.body.segments.iter().map(move |seg| {
                (
                    seg.pos,
                    knowledge.is_some_and(|k| k.can_pass_through_other(snake.snake_type, seg)),
                )
            })
        });

        let mut cells = HashMap::new();
        for (pos, passable) in own.chain(others) {
            let obstacle = if passable {
                Obstacle::Passable
            } else {
                Obstacle::Blocked
            };
            // several segments can share a cell: the worst of them counts
            cells
                .entry(pos)
                .and_modify(|other: &mut Obstacle| *other = (*other).max(obstacle))
                .or_insert(obstacle);
        }
        Self(cells)
    }

    pub fn at(&self, pos: HexPoint) -> Option<Obstacle> {
        self.0.get(&pos).copied()
    }

    pub fn blocks(&self, pos: HexPoint) -> bool {
        self.at(pos) == Some(Obstacle::Blocked)
    }
}
