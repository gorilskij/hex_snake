use super::{Path, PathFinder};
use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::HexPoint;
use crate::snake::{Body, PassthroughKnowledge};
use crate::view::snakes::Snakes;

pub struct SpaceFilling;

impl PathFinder for SpaceFilling {
    fn get_path(
        &self,
        _targets: &mut dyn Iterator<Item = HexPoint>,
        _body: &Body,
        _passthrough_knowledge: Option<&PassthroughKnowledge>,
        _other_snakes: &dyn Snakes,
        _gtx: &GameContext,
    ) -> Option<Path> {
        // TODO: implement
        None
    }
}
