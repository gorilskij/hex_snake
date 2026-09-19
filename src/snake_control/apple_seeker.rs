use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::{Dir, HexDim};
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::snake_control::pathfinder::{Obstacles, Path, PathFinder};
use crate::snake_control::Controller;
use crate::view::snakes::Snakes;

/// Seeks apples by pathfinding to the nearest one (using the wrapped
/// [`PathFinder`] strategy) and following the resulting path.
pub struct AppleSeeker {
    pub pathfinder: Box<dyn PathFinder + Send + Sync>,
    // implicitly, the target is always the last cell in the path; paired with
    // the board it was planned on, since steps across the edge only connect
    // on a board of that size
    pub path: Option<(Path, HexDim)>,
}

impl AppleSeeker {
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
            None => true,
            Some((path, _)) if path.is_empty() => true,
            // the board was resized
            Some((_, board_dim)) if *board_dim != gtx.board_dim => true,
            Some((path, _)) => 'arm: {
                // recalculate if we're not following the path
                let head = body.segments[0].pos;
                if head == path[0] {
                } else if path.len() >= 2 && head == path[1] {
                    path.pop_front();
                } else {
                    // strayed off the path (or the path is too short to
                    // still be following it) -> recalculate
                    break 'arm true;
                }

                // recalculate if the next step doesn't connect (e.g. the head
                // was moved some other way)
                if path.len() >= 2 && path[0].single_step_dir_to(path[1], gtx.board_dim).is_none() {
                    break 'arm true;
                }

                // recalculate if something moved into the way (the head
                // itself is path[0])
                let obstacles = Obstacles::new(body, knowledge, other_snakes);
                if path.iter().skip(1).any(|&pos| obstacles.blocks(pos)) {
                    break 'arm true;
                }

                // recalculate if the target isn't there anymore (always, for
                // a backup path that doesn't lead to an apple)
                let target = *path.back().unwrap();
                !apples.iter().any(|apple| apple.pos == target)
            }
        };

        if recalculate_path {
            // find the shortest path to any apple and lock in that apple as the target
            self.path = self
                .pathfinder
                .get_path(&apples, body, knowledge, other_snakes, gtx)
                .map(|path| (path, gtx.board_dim));

            if self.path.is_none() {
                println!("failed to find path");
                println!("apples: {}", apples.len());
            }
        }
    }
}

impl Controller for AppleSeeker {
    fn next_dir(
        &mut self,
        body: &mut Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
    ) -> Option<Dir> {
        self.recalculate_path(body, knowledge, other_snakes, apples, gtx);

        // TODO: detect and warn about excessive recalculation
        // WARNING: this can cause excessive recalculation
        let (path, _) = self.path.as_ref()?;
        if path.len() < 2 {
            // if the path has length 1, we're about to eat an apple, maintain course
            return Some(body.dir);
        }

        // not `dir_to`: across the board's edge that points the opposite way.
        // A freshly (re)calculated path always connects.
        path[0].single_step_dir_to(path[1], gtx.board_dim)
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
        self.path.as_ref().map(|(path, _)| path)
    }

    fn reset(&mut self, _dir: Dir) {
        self.path = None;
    }
}
