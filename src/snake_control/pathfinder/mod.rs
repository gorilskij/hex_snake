mod space_filling;
mod weighted_bfs;
mod with_backup;

use crate::app::game_context::GameContext;
use crate::basic::{Dir, HexPoint};
use crate::snake::Body;
use crate::view::snakes::Snakes;
use std::collections::VecDeque;

use crate::snake::eat_mechanics::Knowledge;
use crate::view::targets::Targets;
use space_filling::SpaceFilling;
use weighted_bfs::WeightedBFS;
use with_backup::WithBackup;

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
    WeightedBFS,
    SpaceFilling,
    WithBackup {
        main: Box<Template>,
        backup: Box<Template>,
    },
}

impl Template {
    pub fn into_pathfinder(self, _start_dir: Dir) -> Box<dyn PathFinder + Send + Sync> {
        match self {
            Template::WeightedBFS => Box::new(WeightedBFS),
            Template::SpaceFilling => Box::new(SpaceFilling),
            Template::WithBackup { main, backup } => Box::new(WithBackup {
                main: main.into_pathfinder(_start_dir),
                backup: backup.into_pathfinder(_start_dir),
            }),
        }
    }
}
