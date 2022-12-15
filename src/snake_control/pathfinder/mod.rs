mod algorithm1;
mod space_filling;
mod with_backup;

use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::{Dir, HexPoint};
use crate::snake::{Body, PassthroughKnowledge};
use crate::view::snakes::Snakes;
use std::collections::VecDeque;

use algorithm1::Algorithm1;
use space_filling::SpaceFilling;
use with_backup::WithBackup;

// TODO: factor this out
pub type Path = VecDeque<HexPoint>;

pub trait PathFinder {
    // TODO: pass target or list of targets as arguments
    //  (essentially replace the apples argument)
    fn get_path(
        &self,
        body: &Body,
        passthrough_knowledge: Option<&PassthroughKnowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
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
