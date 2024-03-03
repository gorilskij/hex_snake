use ggez::Context;

use crate::app::fps_control::FpsContext;
use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::{Dir, HexPoint};
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::snake_control::pathfinder::{Path, PathFinder};
use crate::snake_control::Controller;
use crate::view::snakes::Snakes;
use crate::view::targets::Targets;

// TODO: rename to something more descriptive like apple seeker
pub struct Algorithm {
    pub pathfinder: Box<dyn PathFinder + Send + Sync>,
    // implicitly, the target is always the last cell in the path
    pub path: Option<Path>,
}

impl Algorithm {
    fn recalculate_path(
        &mut self,
        body: &Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
    ) {
        // recalculate the path if there is no target of if the last target isn't there anymore
        let recalculate_path = match &mut self.path {
            None => {
                println!("recalculate: no path");
                true
            },
            Some(path) if path.is_empty() => {
                println!("recalculate: path is empty");
                true
            },
            Some(path) => 'arm: {
                // recalculate if we're not following the path
                let head = body.segments[0].pos;
                if head == path[0] {
                } else if head == path[1] {
                    path.pop_front();
                } else {
                    println!("recalculate: not following path");
                    break 'arm true;
                }

                // recalculate if the target isn't there anymore
                let target = *path.back().unwrap();
                if apples.iter().any(|apple| apple.pos == target) {
                    false
                } else {
                    println!("recalculate: target isn't there");
                    true
                }
            },
        };

        if recalculate_path {
            // find the shortest path to any apple and lock in that apple as the target
            self.path = self.pathfinder.get_path(&apples, body, knowledge, other_snakes, gtx);
        }
    }
}

impl Controller for Algorithm {
    fn next_dir(
        &mut self,
        body: &mut Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
        _ftx: &FpsContext,
        _ctx: &Context,
    ) -> Option<Dir> {
        self.recalculate_path(body, knowledge, other_snakes, apples, gtx);

        // TODO: detect and warn about excessive recalculation
        // WARNING: this can cause excessive recalculation
        let path = self.path.as_mut()?;
        if path.len() < 2 {
            // if the path has length 1, we're about to eat an apple, maintain course
            return Some(body.dir);
        }

        let dir = path[0]
            .dir_to(path[1])
            .expect("failed to compute dir between path points");
        Some(dir)
    }

    fn get_path(
        &mut self,
        body: &Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
    ) -> Option<&Path> {
        self.recalculate_path(body, knowledge, other_snakes, apples, gtx);
        self.path.as_ref()
    }

    fn reset(&mut self, _dir: Dir) {
        self.path = None;
    }
}
