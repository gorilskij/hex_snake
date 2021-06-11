// This file contains structs describing various aspects of a segment

use crate::{
    app::snake::palette::SegmentStyle,
    basic::{CellDim, Dir, DrawStyle, Point},
};

// A full (solid) segment starts at 0. and ends at 1.
pub struct SegmentFraction {
    pub start: f32,
    pub end: f32,
}

impl SegmentFraction {
    pub fn solid() -> Self {
        Self { start: 0., end: 1. }
    }

    pub fn appearing(frame_frac: f32) -> Self {
        assert!(
            (0. ..=1.).contains(&frame_frac),
            "Invalid frame-frac {}",
            frame_frac
        );
        Self { start: 0., end: frame_frac }
    }

    pub fn disappearing(frame_frac: f32) -> Self {
        assert!(
            (0. ..=1.).contains(&frame_frac),
            "Invalid frame-frac {}",
            frame_frac
        );
        Self { start: frame_frac, end: 1. }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TurnDirection {
    Clockwise,
    CounterClockwise,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TurnType {
    Straight,
    Blunt(TurnDirection),
    Sharp(TurnDirection),
}

/// Describes a segment in a snake, coming_from is relative to
/// the current cell so TurnDescription { coming_from: D, going_to == U }
/// describes a straight segment coming from below and going up.
/// This means that if coming_from == going_to, the state is invalid.
#[derive(Copy, Clone)]
pub struct TurnDescription {
    pub coming_from: Dir,
    pub going_to: Dir,
}

impl TurnDescription {
    pub fn turn_type(self) -> TurnType {
        use TurnDirection::*;
        use TurnType::*;

        match self.coming_from.clockwise_distance_to(self.going_to) {
            0 => panic!("180° turn: {:?} => {:?}", self.coming_from, self.going_to),
            1 => Sharp(CounterClockwise),
            2 => Blunt(CounterClockwise),
            3 => Straight,
            4 => Blunt(Clockwise),
            5 => Sharp(Clockwise),
            _ => unreachable!(),
        }
    }
}

pub struct SegmentDescription {
    pub destination: Point,
    pub turn: TurnDescription,
    pub fraction: SegmentFraction,
    pub draw_style: DrawStyle,
    pub segment_style: SegmentStyle,
    pub cell_dim: CellDim,
}
