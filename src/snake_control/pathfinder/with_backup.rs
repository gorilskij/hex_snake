use super::{Committed, Goals, Leg, PathFinder, Start};
use crate::app::game_context::GameContext;
use crate::snake::Body;
use crate::snake::eat_mechanics::Knowledge;
use crate::view::snakes::Snakes;

pub struct WithBackup {
    pub main: Box<dyn PathFinder + Send + Sync>,
    pub backup: Box<dyn PathFinder + Send + Sync>,
}

impl PathFinder for WithBackup {
    fn get_path(
        &self,
        start: Start,
        goals: &Goals,
        committed: Committed,
        body: &Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        gtx: &GameContext,
    ) -> Option<Leg> {
        // first try the main pathfinder, if that fails, fall back to the backup
        // pathfinder, whose leg has no target and so marks the plan a fallback
        self.main
            .get_path(start, goals, committed, body, knowledge, other_snakes, gtx)
            .or_else(|| {
                self.backup
                    .get_path(start, goals, committed, body, knowledge, other_snakes, gtx)
            })
    }
}
