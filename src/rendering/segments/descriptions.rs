// This file contains structs describing various aspects of a segment

use crate::basic::{CellDim, Dir, Point};
use crate::color::Color;
use crate::rendering::segments::smooth_segments::SubsegmentIdx;
use crate::snake::palette::SegmentStyle;
use crate::snake::{SegmentId, SegmentType, ZIndex};
use crate::{rendering, snake};

// A full (solid) segment starts at 0. and ends at 1.
#[derive(Copy, Clone, Debug)]
pub struct SegmentFraction {
    pub start: f32,
    pub end: f32,
}

impl SegmentFraction {
    pub fn solid() -> Self {
        Self { start: 0., end: 1. }
    }

    pub fn appearing(frame_fraction: f32) -> Self {
        assert!(
            (0. ..=1.).contains(&frame_fraction),
            "Invalid frame fraction: {frame_fraction}",
        );
        Self { start: 0., end: frame_fraction }
    }

    pub fn disappearing(frame_fraction: f32) -> Self {
        assert!(
            (0. ..=1.).contains(&frame_fraction),
            "Invalid frame fraction: {frame_fraction}",
        );
        Self { start: frame_fraction, end: 1. }
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
#[derive(Copy, Clone, Debug)]
pub struct TurnDescription {
    pub coming_from: Dir,
    pub going_to: Dir,
    pub fraction: f32,
}

// impl Default for TurnDescription {
//     fn default() -> Self {
//         Self {
//             coming_from: Dir::D,
//             going_to: Dir::U,
//         }
//     }
// }

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SegmentLocation {
    Head, // first segment
    Body, // somewhere inbetween
    Tail, // last segment
}

impl SegmentLocation {
    pub fn new(segment_idx: usize, body: &snake::Body) -> Self {
        match segment_idx {
            0 => Self::Head,
            i if i == body.visible_len() - 1 && body.grow == 0 => Self::Tail,
            _ => Self::Body,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SegmentDescription {
    pub segment_id: SegmentId,
    pub location: SegmentLocation,
    pub destination: Point,
    pub turn: TurnDescription,
    pub fraction: SegmentFraction,
    /// `SegmentFraction` of the previous segment (towards the head)
    pub prev_fraction: Option<SegmentFraction>,
    pub draw_style: rendering::Style,
    pub segment_type: SegmentType,
    pub segment_style: SegmentStyle,
    pub z_index: ZIndex,
    pub cell_dim: CellDim,
}

#[derive(Debug, Copy, Clone)]
pub enum RoundHeadDescription {
    /// Only the beginning of the round head is within the current segment
    Tip { segment_end: f32 },
    /// The full round head is within the current segment
    Full { segment_end: f32 },
    /// Only the end of the round head is within the current segment
    Tail { prev_segment_end: f32 },
    /// No part of the round head is within this segment
    Gone,
}

impl SegmentFraction {
    pub fn round_head_description(self, prev: Option<Self>, cell_dim: CellDim) -> RoundHeadDescription {
        let height = cell_dim.height();
        let head_radius = cell_dim.side / 2.;

        use RoundHeadDescription::*;
        match prev {
            None if self.end * height < head_radius => Tip { segment_end: self.end },
            None => Full { segment_end: self.end },
            Some(prev) if prev.end * height < head_radius => Tail { prev_segment_end: prev.end },
            Some(_) => Gone,
        }
    }
}

pub struct Polygon {
    pub subsegment_idx: SubsegmentIdx,
    pub points: Vec<Point>,
    pub color: Color,
    // to prevent direct construction
    _private: (),
}

impl Polygon {
    pub fn new(subsegment_idx: SubsegmentIdx, points: Vec<Point>, color: Color) -> Option<Self> {
        (points.len() >= 3).then(|| Self {
            subsegment_idx,
            points,
            color,
            _private: (),
        })
    }
}
