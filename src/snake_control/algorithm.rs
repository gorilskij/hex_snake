use crate::app::game_context::GameContext;
use crate::app::guidance::{Path, PathFinder};
use crate::apple::Apple;
use crate::basic::Dir;
use crate::snake::{Body, PassthroughKnowledge};
use crate::snake_control::Controller;
use crate::view::snakes::Snakes;
use ggez::Context;

pub struct Algorithm {
    pub pathfinder: Box<dyn PathFinder + Send + Sync>,
    pub path: Option<Path>,
}

impl Algorithm {
    fn recalculate_path(
        &mut self,
        body: &Body,
        passthrough_knowledge: Option<&PassthroughKnowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
    ) {
        if let Some(path) = &mut self.path {
            if path.len() > 1 {
                let head_pos = body.segments[0].pos;

                if path[0] == head_pos {
                    // following path
                    return;
                }

                if path.len() > 1 && path[1] == head_pos {
                    // following path, remove passed cell
                    path.pop_front();
                    return;
                }
            }
        }

        // recalculate
        self.path =
            Some(
                self.pathfinder
                    .get_path(body, passthrough_knowledge, other_snakes, apples, gtx),
            );
    }
}

impl Controller for Algorithm {
    fn next_dir(
        &mut self,
        body: &mut Body,
        passthrough_knowledge: Option<&PassthroughKnowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
        _ctx: &Context,
    ) -> Option<Dir> {
        self.recalculate_path(body, passthrough_knowledge, other_snakes, apples, gtx);

        let path = self.path.as_mut().unwrap();
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
        passthrough_knowledge: Option<&PassthroughKnowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
    ) -> Option<&Path> {
        self.recalculate_path(body, passthrough_knowledge, other_snakes, apples, gtx);
        self.path.as_ref()
    }

    fn reset(&mut self, _dir: Dir) {
        self.path = None;
    }
}
