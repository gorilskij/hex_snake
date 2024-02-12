use crate::app::fps_control::FpsContext;
use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::{angle_distance, CellDim, Dir, HexDim, HexPoint};
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::{self, Body, Segment};
use crate::snake_control::Controller;
use crate::support::partial_min_max::PartialMinMax;
use crate::view::snakes::Snakes;
use ggez::Context;
use std::f32::consts::TAU;

// tries to kill player
pub struct Killer;

// potential alternative to searching for the closest angle
// would still require searching through a list so not an improvement..
// fn round_angle_to_closest_dir(angle: f32) -> Dir {
//     use std::f32::consts::{FRAC_PI_3, FRAC_PI_6};
//     let rounded_angle = (angle / FRAC_PI_3).floor() * FRAC_PI_3 + FRAC_PI_6;
// }

fn rough_direction(
    from: HexPoint,
    to: HexPoint,
    body: &Body,
    other_snakes: impl Snakes,
    board_dim: HexDim,
) -> Option<Dir> {
    // dy is scaled to convert from 'hex' coordinates to approximate cartesian coordinates
    let CellDim { sin, .. } = CellDim::from(1.);
    let dx = (to.h - from.h) as f32;
    let dy = -(to.v - from.v) as f32 / (2. * sin);
    let angle = (dy.atan2(dx) + TAU) % TAU;

    let head_pos = body.segments[0].pos;

    // this could probably be done with math
    Dir::ANGLES
        .iter()
        .copied()
        .filter(|(d, _)| *d != -body.dir)
        .filter(|(d, _)| {
            distance_to_snake(
                head_pos,
                *d,
                body,
                &other_snakes as &dyn Snakes,
                board_dim,
                Some(2),
            ) > 1
        })
        .partial_min_by_key(|(_, a)| angle_distance(angle, *a))
        // .take()
        .map(|(d, _)| d)
}

fn distance_to_snake(
    mut point: HexPoint,
    dir: Dir,
    body: &Body,
    other_snakes: impl Snakes,
    board_dim: HexDim,
    max_dist: Option<usize>, // if not within max_dist, returns max_dist
) -> usize {
    // guaranteed to terminate anyway whenever the head reaches itself again
    let upper_bound = max_dist.unwrap_or(usize::MAX);
    for distance in 1..=upper_bound {
        point = point.wrapping_translate(dir, 1, board_dim);

        for Segment { pos, .. } in body.segments.iter().chain(other_snakes.iter_segments()) {
            if *pos == point {
                return distance;
            }
        }
    }
    upper_bound
}

impl Controller for Killer {
    fn next_dir(
        &mut self,
        body: &mut Body,
        _: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        _apples: &[Apple],
        gtx: &GameContext,
        _ftx: &FpsContext,
        _ctx: &Context,
    ) -> Option<Dir> {
        let player_snake = other_snakes
            .iter()
            .filter(|s| s.snake_type == snake::Type::Player)
            .min_by_key(|s| s.head().pos.manhattan_distance(body.segments[0].pos))
            .expect("no player snake found");

        let mut target = player_snake.head().pos;
        // how many cells ahead of the player to target
        for _ in 0..1 {
            target = target.wrapping_translate(player_snake.body.dir, 1, gtx.board_dim);
        }
        rough_direction(
            body.segments[0].pos,
            target,
            body,
            other_snakes,
            gtx.board_dim,
        )
    }
}
