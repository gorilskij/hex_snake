use super::{Path, PathFinder};
use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::HexPoint;
use crate::snake::{Body, PassthroughKnowledge};
use crate::view::snakes::Snakes;

pub struct WithBackup {
    pub main: Box<dyn PathFinder + Send + Sync>,
    pub backup: Box<dyn PathFinder + Send + Sync>,
}

impl PathFinder for WithBackup {
    fn get_path(
        &self,
        targets: &mut dyn Iterator<Item = HexPoint>,
        body: &Body,
        passthrough_knowledge: Option<&PassthroughKnowledge>,
        other_snakes: &dyn Snakes,
        gtx: &GameContext,
    ) -> Option<Path> {
        let targets: Vec<_> = targets.collect();

        // first try the main pathfinder, if that fails, fall back to the backup pathfinder
        self.main
            .get_path(
                &mut targets.iter().copied(),
                body,
                passthrough_knowledge,
                other_snakes,
                gtx,
            )
            .or_else(|| {
                self.backup.get_path(
                    &mut targets.iter().copied(),
                    body,
                    passthrough_knowledge,
                    other_snakes,
                    gtx,
                )
            })
    }
}
