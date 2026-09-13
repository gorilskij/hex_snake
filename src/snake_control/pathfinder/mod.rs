mod space_filling;
mod weighted_bfs;
mod with_backup;

use std::collections::{HashMap, VecDeque};

use space_filling::SpaceFilling;
use weighted_bfs::WeightedBFS;
pub use weighted_bfs::Weights;
use with_backup::WithBackup;

use crate::app::game_context::GameContext;
use crate::basic::{Dir, HexPoint};
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::view::snakes::Snakes;
use crate::view::targets::Targets;

pub type Path = VecDeque<HexPoint>;

pub trait PathFinder {
    fn get_path(
        &self,
        targets: &dyn Targets,
        body: &Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        gtx: &GameContext,
    ) -> Option<Path>;
}

#[derive(Clone, Debug)]
pub enum Template {
    WeightedBFS(Weights),
    SpaceFilling,
    WithBackup { main: Box<Template>, backup: Box<Template> },
}

impl Template {
    pub fn into_pathfinder(self, _start_dir: Dir) -> Box<dyn PathFinder + Send + Sync> {
        match self {
            Template::WeightedBFS(weights) => Box::new(WeightedBFS { weights }),
            Template::SpaceFilling => Box::new(SpaceFilling),
            Template::WithBackup { main, backup } => Box::new(WithBackup {
                main: main.into_pathfinder(_start_dir),
                backup: backup.into_pathfinder(_start_dir),
            }),
        }
    }
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
            snake
                .body
                .segments
                .iter()
                .map(move |seg| (seg.pos, knowledge.is_some_and(|k| k.can_pass_through_other(snake.snake_type, seg))))
        });

        let mut cells = HashMap::new();
        for (pos, passable) in own.chain(others) {
            let obstacle = if passable { Obstacle::Passable } else { Obstacle::Blocked };
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
