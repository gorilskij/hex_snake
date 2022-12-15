use super::{Path, PathFinder};
use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::snake::{Body, PassthroughKnowledge};
use crate::view::snakes::Snakes;

pub struct SpaceFilling;

impl PathFinder for SpaceFilling {
    fn get_path(
        &self,
        body: &Body,
        passthrough_knowledge: Option<&PassthroughKnowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
    ) -> Option<Path> {
        // TODO: implement
        None
    }
}
