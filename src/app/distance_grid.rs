use std::cmp::{max, min};
use std::collections::HashSet;
use std::mem;
use ggez::Context;
use ggez::graphics::{DrawMode, Mesh, MeshBuilder};
use itertools::Itertools;
use crate::app::app_error::AppResult;
use crate::app::game_context::GameContext;
use crate::app::rendering::segments::render_hexagon;
use crate::app::snake::Snake;
use crate::app::snake::utils::OtherSnakes;
use crate::basic::{CellDim, Dir, HexDim, HexPoint};
use crate::basic::transformations::translate;
use crate::color::Color;

type Distance = usize;

struct Iter {
    board_dim: HexDim,

    // -- bfs --
    // also used to store positions occupied by snakes
    seen: HashSet<HexPoint>,
    occupied: HashSet<HexPoint>,
    // all the positions in a generation have the same distance
    dist: Distance,
    // search will only continue from generation_alive
    generation_alive: Vec<HexPoint>,
    generation_dead: Vec<HexPoint>,
    // once a new generation is computed, it's iterated over to
    // return the values one by one
    output_idx: usize,
}

impl Iterator for Iter {
    type Item = (HexPoint, Distance);

    fn next(&mut self) -> Option<Self::Item> {
        let num_alive = self.generation_alive.len();
        let num_dead = self.generation_dead.len();

        if self.output_idx < num_alive {
            let ret = self.generation_alive[self.output_idx];
            self.output_idx += 1;
            Some((ret, self.dist))
        } else if self.output_idx < num_alive + num_dead {
            let ret = self.generation_dead[self.output_idx - num_alive];
            self.output_idx += 1;
            Some((ret, self.dist))
        } else {
            // bfs step
            let board_dim = self.board_dim;

            self.generation_dead = vec![];
            let generation_alive = mem::replace(&mut self.generation_alive, vec![]);

            generation_alive
                .into_iter()
                .flat_map(move |pos| {
                    Dir::iter()
                        .map(move |dir| pos.wrapping_translate(dir, 1, board_dim))
                })
                .filter(|new_pos| !self.seen.contains(new_pos))
                .sorted_unstable()
                .dedup()
                .for_each(|pos| {
                    if self.occupied.contains(&pos) {
                        self.generation_dead.push(pos)
                    } else {
                        self.generation_alive.push(pos)
                    }
                });

            if self.generation_alive.is_empty() {
                None
            } else {
                self.seen.extend(&self.generation_alive);
                self.seen.extend(&self.generation_dead);
                self.dist += 1;
                self.output_idx = 1;
                Some((self.generation_alive[0], self.dist))
            }
        }
    }
}

fn find_distances(
    player_snake: &Snake,
    other_snakes: OtherSnakes,
    board_dim: HexDim,
) -> Iter {
    let occupied = if let Some(passthrough_knowledge) = player_snake.controller.passthrough_knowledge() {
        player_snake.body.cells
            .iter()
            .chain(other_snakes.iter_segments())
            .filter(|seg| !passthrough_knowledge.can_pass_through_self(seg))
            .map(|seg| seg.pos)
            .collect()
    } else {
        player_snake.body.cells
            .iter()
            .chain(other_snakes.iter_segments())
            .map(|seg| seg.pos)
            .collect()
    };

    // setup bfs
    Iter {
        board_dim,
        seen: HashSet::new(),
        occupied,
        dist: 0,
        generation_alive: vec![player_snake.head().pos],
        generation_dead: vec![],
        output_idx: 1, // trigger bfs step immediately
    }
}

pub fn mesh(
    snake: &Snake,
    other_snakes: OtherSnakes,
    ctx: &mut Context,
    gtx: &GameContext,
) -> AppResult<Mesh> {
    // not actually max distance but a good estimate, anything
    // higher gets the same color
    let max_dist = max(gtx.board_dim.h, gtx.board_dim.v) as f64 / 2.0;
    let mid_dist = max_dist as f64 / 2.0;

    let mut builder = MeshBuilder::new();
    find_distances(snake, other_snakes, gtx.board_dim)
        .map(|(pos, dist)| {
            const ALPHA: f32 = 0.3;
            const CLOSEST_COLOR: Color = Color::from_rgb(51, 204, 51).with_alpha(ALPHA);
            const MIDWAY_COLOR: Color = Color::from_rgb(255, 255, 0).with_alpha(ALPHA);
            const FARTHEST_COLOR: Color = Color::from_rgb(204, 0, 0).with_alpha(ALPHA);

            let dist = dist as f64;

            let mut dist_ratio = dist / max_dist;
            if dist_ratio > 1.0 {
                dist_ratio = 1.0;
            }
            let color = if dist_ratio < 0.5 {
                let ratio = dist_ratio * 2.0;
                (1. - ratio) * CLOSEST_COLOR + ratio * MIDWAY_COLOR
            } else {
                let ratio = dist_ratio * 2.0 - 1.0;
                (1. - ratio) * MIDWAY_COLOR + ratio * FARTHEST_COLOR
            };

            let mut hexagon = render_hexagon(gtx.cell_dim);
            translate(&mut hexagon, pos.to_cartesian(gtx.cell_dim));
            builder.polygon(DrawMode::fill(), &hexagon, *color).map(|_| ())
        })
        .collect::<Result<(), _>>()?;
    Ok(builder.build(ctx)?)
}
