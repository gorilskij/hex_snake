mod algorithm1;
mod with_backup;
mod space_filling;

use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::{Dir, HexPoint};
use crate::snake::{Body, PassthroughKnowledge};
use crate::view::snakes::Snakes;
use map_with_state::IntoMapWithState;
use std::cell::RefCell;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

use algorithm1::Algorithm1;
use with_backup::WithBackup;
use space_filling::SpaceFilling;

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
    pub fn into_pathfinder(self, start_dir: Dir) -> Box<dyn PathFinder + Send + Sync> {
        match self {
            Template::Algorithm1 => Box::new(Algorithm1),
            Template::SpaceFilling => Box::new(SpaceFilling),
            Template::WithBackup { main, backup } => Box::new(WithBackup { main: main.into_pathfinder(start_dir), backup: backup.into_pathfinder(start_dir) }),
        }
    }
}
