use super::{Path, PathFinder};
use crate::app::game_context::GameContext;
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::view::snakes::Snakes;
use crate::view::targets::Targets;

pub struct SpaceFilling;

impl PathFinder for SpaceFilling {
    fn get_path(
        &self,
        _targets: &dyn Targets,
        _body: &Body,
        _passthrough_knowledge: Option<&Knowledge>,
        _other_snakes: &dyn Snakes,
        _gtx: &GameContext,
    ) -> Option<Path> {
        // TODO: implement
        None
    }
}
