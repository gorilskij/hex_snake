use crate::{
    app::{
        screen::Apple,
        snake::{
            controller::{angle_distance, Controller, OtherSnakes},
            Body, Segment, Type,
        },
    },
    basic::{CellDim, Dir, HexDim, HexPoint},
    partial_min_max::PartialMinMax,
};
use std::f32::consts::PI;

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
    other_snakes: OtherSnakes,
    board_dim: HexDim,
) -> Option<Dir> {
    const TWO_PI: f32 = 2. * PI;

    // dy is scaled to convert from 'hex' coordinates to approximate cartesian coordinates
    let CellDim { sin, .. } = CellDim::from(1.);
    let dx = (to.h - from.h) as f32;
    let dy = -(to.v - from.v) as f32 / (2. * sin);
    let angle = (dy.atan2(dx) + TWO_PI) % TWO_PI;

    let head_pos = body.cells[0].pos;

    // this could probably be done with math
    Dir::ANGLES
        .iter()
        .copied()
        .filter(|(d, _)| *d != -body.dir)
        .filter(|(d, _)| {
            distance_to_snake(head_pos, *d, body, other_snakes, board_dim, Some(2)) > 1
        })
        .partial_min_by_key(|(_, a)| angle_distance(angle, *a))
        .take()
        .map(|(d, _)| d)
}

fn distance_to_snake(
    mut point: HexPoint,
    dir: Dir,
    body: &Body,
    other_snakes: OtherSnakes,
    board_dim: HexDim,
    max_dist: Option<usize>, // if not within max_dist, returns max_dist
) -> usize {
    // guaranteed to terminate anyway whenever the head reaches itself again
    let upper_bound = max_dist.unwrap_or(usize::MAX);
    for distance in 1..=upper_bound {
        point = point.wrapping_translate(dir, 1, board_dim);

        for Segment { pos, .. } in body.cells.iter().chain(other_snakes.iter_segments()) {
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
        other_snakes: OtherSnakes,
        _apples: &[Apple],
        board_dim: HexDim,
    ) -> Option<Dir> {
        let player_snake = other_snakes
            .iter_snakes()
            .filter(|s| s.snake_type == Type::Player)
            .min_by_key(|s| s.head().pos.manhattan_distance(body.cells[0].pos))
            .expect("no player snake found");

        let mut target = player_snake.head().pos;
        // how many cells ahead of the player to target
        for _ in 0..1 {
            target = target.wrapping_translate(player_snake.dir(), 1, board_dim);
        }
        rough_direction(body.cells[0].pos, target, body, other_snakes, board_dim)
    }
}
