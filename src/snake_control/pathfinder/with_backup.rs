use super::{Path, PathFinder};
use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::snake::{Body, PassthroughKnowledge};
use crate::view::snakes::Snakes;

pub struct WithBackup {
    pub main: Box<dyn PathFinder + Send + Sync>,
    pub backup: Box<dyn PathFinder + Send + Sync>,
}

impl PathFinder for WithBackup {
    fn get_path(
        &self,
        body: &Body,
        passthrough_knowledge: Option<&PassthroughKnowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
    ) -> Option<Path> {
        // first try the main pathfinder, if that fails, fall back to the backup pathfinder
        self.main
            .get_path(body, passthrough_knowledge, other_snakes, apples, gtx)
            .or_else(|| {
                self.backup
                    .get_path(body, passthrough_knowledge, other_snakes, apples, gtx)
            })
    }
}
