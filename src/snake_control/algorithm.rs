use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::{Dir, HexPoint};
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::snake_control::pathfinder::{Path, PathFinder};
use crate::snake_control::Controller;
use crate::support::limits::Limits;
use crate::view::snakes::Snakes;
use ggez::Context;

// TODO: rename to something more descriptive like apple seeker
pub struct Algorithm {
    pub pathfinder: Box<dyn PathFinder + Send + Sync>,
    pub path: Option<Path>,

    // Also store the expected index in the apples array, chances are it didn't change
    pub current_target: Option<(usize, HexPoint)>,
}

impl Algorithm {
    fn recalculate_path(
        &mut self,
        body: &Body,
        passthrough_knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
    ) {
        match self.current_target {
            Some((i, target)) if apples[i].pos == target => {},
            Some((_, target)) if let Some((i, _)) =
                apples
                    .iter()
                    .enumerate()
                    .find(|(_, apple)| apple.pos == target) => {
                self.current_target.as_mut().unwrap().0 = i;
            }
            _ => self.path = None,
        }

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

        // TODO: don't recalculate until called upon (lazy ai)
        // recalculate
        println!("recalculating");
        self.path =
            self.pathfinder
                .get_path(&apples, body, passthrough_knowledge, other_snakes, gtx);

        // assign wrong index 0, the true index will be found in the next iteration
        // (it's only cache anyway)
        self.current_target = self
            .path
            .as_ref()
            .and_then(|path| path.last().copied())
            .map(|pos| (0, pos));
    }
}

impl Controller for Algorithm {
    fn next_dir(
        &mut self,
        body: &mut Body,
        passthrough_knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
        _ctx: &Context,
    ) -> Option<Dir> {
        self.recalculate_path(body, passthrough_knowledge, other_snakes, apples, gtx);

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
        passthrough_knowledge: Option<&Knowledge>,
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
