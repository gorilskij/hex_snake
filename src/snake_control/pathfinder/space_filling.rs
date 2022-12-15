use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::rc::Rc;
use map_with_state::IntoMapWithState;
use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::{Dir, HexPoint};
use crate::snake::{Body, PassthroughKnowledge};
use crate::view::snakes::Snakes;
use super::{Path, PathFinder};

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
        return None
    }
}
