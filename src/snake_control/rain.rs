use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::Dir;
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::{self, Body};
use crate::snake_control::Controller;
use crate::view::snakes::Snakes;

/// Falls straight down, around anything that isn't rain, until it disappears
/// at the bottom of the board (see `advance_snakes`).
pub struct Rain;

impl Controller for Rain {
    fn next_dir(
        &mut self,
        body: &mut Body,
        _: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        _: &[Apple],
        gtx: &GameContext,
    ) -> Option<Dir> {
        let head = body.segments[0].pos;
        let free = |dir: Dir| {
            let next = head.wrapping_translate(dir, 1, gtx.board_dim);
            !other_snakes
                .iter()
                .filter(|s| s.snake_type != snake::Type::Rain)
                .flat_map(|s| s.body.segments.iter())
                .any(|segment| segment.pos == next)
        };

        // down if possible, else down left or down right (either first, at
        // random), else crash
        let sides = if rand::random() {
            [Dir::Dl, Dir::Dr]
        } else {
            [Dir::Dr, Dir::Dl]
        };
        std::iter::once(Dir::D).chain(sides).find(|&dir| free(dir))
    }
}
