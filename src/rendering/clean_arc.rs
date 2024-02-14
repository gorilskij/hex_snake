use std::iter;

use itertools::chain;
use lyon_geom::{Angle, Arc};

use crate::basic::Point;

#[derive(Copy, Clone, Debug)]
pub struct CleanArc {
    pub center: Point,
    pub radius: f32,
    pub start_angle: f32,
    pub end_angle: f32,
}

impl CleanArc {
    fn extreme_points(self) -> (Point, Point) {
        // TODO: probably cheaper to calculate without rotation
        let Self {
            center,
            radius,
            start_angle,
            end_angle,
        } = self;
        let starting_point = center + Point { x: radius, y: 0.0 };
        let first_point = starting_point.rotate_clockwise(center, start_angle);
        let last_point = starting_point.rotate_clockwise(center, end_angle);
        (first_point, last_point)
    }

    pub fn flattened(&self, tolerance: f32) -> impl Iterator<Item = Point> {
        let (first, last) = self.extreme_points();

        let arc = Arc {
            center: self.center.into(),
            radii: Point::square(self.radius).into(),
            start_angle: Angle { radians: self.start_angle },
            sweep_angle: Angle {
                radians: self.end_angle - self.start_angle,
            },
            x_rotation: Angle { radians: 0. },
        };

        chain!(
            iter::once(first),
            arc.flattened(tolerance).map(Into::into),
            iter::once(last),
        )
    }
}
