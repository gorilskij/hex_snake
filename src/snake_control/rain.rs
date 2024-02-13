use crate::app::fps_control::FpsContext;
use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::Dir;
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::{self, Body};
use crate::snake_control::Controller;
use crate::view::snakes::Snakes;
use ggez::Context;

pub struct Rain;

impl Controller for Rain {
    fn next_dir(
        &mut self,
        body: &mut Body,
        _: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        _: &[Apple],
        gtx: &GameContext,
        _ftx: &FpsContext,
        _ctx: &Context,
    ) -> Option<Dir> {
        if body.segments[0].pos.v == gtx.board_dim.v - 1 {
            // todo!("return die")
            eprintln!("TODO: suicide (or even better, disappear)");
            return None;
        }

        // TODO: randomize, make dl and dr equal probability
        // if possible, go down, else try to go down left or down right, else crash
        let next_d = body.segments[0].pos.translate(Dir::D, 1);
        let next_dl = body.segments[0]
            .pos
            .wrapping_translate(Dir::Dl, 1, gtx.board_dim);
        let next_dr = body.segments[0]
            .pos
            .wrapping_translate(Dir::Dr, 1, gtx.board_dim);

        let mut d_occupied = false;
        let mut dl_occupied = false;
        let mut dr_occupied = false;

        other_snakes
            .iter()
            .filter(|s| s.snake_type != snake::Type::Rain)
            .flat_map(|s| s.body.segments.iter().map(|c| c.pos))
            .for_each(|pos| {
                if pos == next_d {
                    d_occupied = true;
                }
                if pos == next_dl {
                    dl_occupied = true;
                }
                if pos == next_dr {
                    dr_occupied = true;
                }
            });

        if !d_occupied {
            Some(Dir::D)
        } else if !dl_occupied {
            Some(Dir::Dl)
        } else if !dr_occupied {
            Some(Dir::Dr)
        } else {
            None
        }
    }
}
