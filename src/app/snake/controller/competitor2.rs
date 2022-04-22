use crate::{
    app::{
        apple::Apple,
        game_context::GameContext,
        snake::{
            controller::{Controller, OtherSnakes},
            Body,
        },
    },
    basic::{angle_distance, CellDim, Dir, Dir12, HexPoint},
    partial_min_max::PartialMinMax,
};
use ggez::Context;
use itertools::Itertools;
use std::f32::consts::PI;

pub struct Competitor2 {
    pub dir_state: bool, // Dir12 flip-flop state
    pub target_apple: Option<HexPoint>,
    pub frames_since_update: usize,
}

impl Competitor2 {
    const UPDATE_EVERY_N_FRAMES: usize = 20;
}

impl Controller for Competitor2 {
    fn next_dir(
        &mut self,
        body: &mut Body,
        other_snakes: OtherSnakes,
        apples: &[Apple],
        gtx: &GameContext,
        _ctx: &Context,
    ) -> Option<Dir> {
        // this also sets the target apple on the first frame
        if self.frames_since_update % Self::UPDATE_EVERY_N_FRAMES == 0 {
            self.target_apple = None;
        }
        self.frames_since_update += 1;

        let head_pos = body.cells[0].pos;
        if let Some(pos) = self.target_apple {
            if pos == head_pos {
                // apple eaten
                self.target_apple = None;
            }
        }

        let target_pos = match self.target_apple {
            None => {
                let closest_apple = apples
                    .iter()
                    .map(|apple| apple.pos)
                    .min_by_key(|pos| head_pos.manhattan_distance(*pos))?;
                self.target_apple = Some(closest_apple);
                closest_apple
            }
            Some(pos) => pos,
        };

        const TWO_PI: f32 = 2. * PI;
        let CellDim { side, sin, cos, .. } = CellDim::from(1.);

        let x_step = side + cos;
        let y_step = 2. * sin;

        let dh = target_pos.h - head_pos.h;
        let dv = target_pos.v - head_pos.v;

        let dx = dh as f32 / x_step;
        let dy = -dv as f32 / y_step; // convert to y going up
        let angle = (dy.atan2(dx) + TWO_PI) % TWO_PI;

        let mut forbidden_directions = body
            .cells
            .iter()
            .chain(other_snakes.iter_segments())
            .map(|seg| seg.pos)
            .filter(|pos| pos.manhattan_distance(head_pos) == 1)
            .map(|pos| {
                head_pos
                    .dir_to(pos)
                    .unwrap_or_else(|| panic!("no direction from {:?} to {:?}", head_pos, pos))
            })
            .collect_vec();
        forbidden_directions.push(-body.dir);

        // let dir_is_safe = |dir: Dir12| {
        //     if dir == Single(-body.dir) {
        //         return false;
        //     }
        //     let translate_dir = dir.to_dir(self.dir_state);
        //     let new_head = head_pos.wrapping_translate(translate_dir, 1, board_dim);
        //     !forbidden_head_positions.contains(&new_head)
        // };

        // this could probably be done with math
        let new_dir = Dir12::ANGLES
            .iter()
            .copied()
            .map(|(d, a)| (d.to_dir(self.dir_state), a))
            .filter(|(d, _)| !forbidden_directions.contains(d))
            .partial_min_by_key(|(_, a)| angle_distance(angle, *a))
            .map(|(d, _)| d);

        // println!("preferred dir: {:?}", dir);
        // println!("target: {:?}", self.target_apple);

        self.dir_state = !self.dir_state;
        new_dir
    }
}
