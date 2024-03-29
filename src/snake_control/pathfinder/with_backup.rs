use super::{Path, PathFinder};
use crate::app::game_context::GameContext;
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::view::snakes::Snakes;
use crate::view::targets::Targets;

pub struct WithBackup {
    pub main: Box<dyn PathFinder + Send + Sync>,
    pub backup: Box<dyn PathFinder + Send + Sync>,
}

impl PathFinder for WithBackup {
    fn get_path(
        &self,
        targets: &dyn Targets,
        body: &Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        gtx: &GameContext,
    ) -> Option<Path> {
        // first try the main pathfinder, if that fails, fall back to the backup pathfinder
        self.main
            .get_path(targets, body, knowledge, other_snakes, gtx)
            .or_else(|| self.backup.get_path(targets, body, knowledge, other_snakes, gtx))
    }
}
