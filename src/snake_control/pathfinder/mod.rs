mod algorithm1;
mod space_filling;
mod with_backup;

use crate::app::game_context::GameContext;
use crate::basic::{Dir, HexPoint};
use crate::snake::Body;
use crate::view::snakes::Snakes;
use std::collections::VecDeque;

use crate::snake::eat_mechanics::Knowledge;
use crate::view::targets::Targets;
use algorithm1::Algorithm1;
use space_filling::SpaceFilling;
use with_backup::WithBackup;

pub type Path = VecDeque<HexPoint>;

pub trait PathFinder {
    fn get_path(
        &self,
        targets: &dyn Targets,
        body: &Body,
        passthrough_knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        gtx: &GameContext,
    ) -> Option<Path>;
}

#[derive(Clone, Debug)]
pub enum Template {
    Algorithm1,
    SpaceFilling,
    WithBackup {
        main: Box<Template>,
        backup: Box<Template>,
    },
}

impl Template {
    pub fn into_pathfinder(self, _start_dir: Dir) -> Box<dyn PathFinder + Send + Sync> {
        match self {
            Template::Algorithm1 => Box::new(Algorithm1),
            Template::SpaceFilling => Box::new(SpaceFilling),
            Template::WithBackup { main, backup } => Box::new(WithBackup {
                main: main.into_pathfinder(_start_dir),
                backup: backup.into_pathfinder(_start_dir),
            }),
        }
    }
}
