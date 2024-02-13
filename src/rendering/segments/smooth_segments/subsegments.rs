use crate::color::Color;
use crate::rendering::segments::descriptions::{SegmentDescription, SegmentFraction};

pub struct Subsegment {
    pub subsegment_idx: usize,
    pub color: Color,
    // start assumed to be the end of the previous subsegment
    // or the start of the parent segment
    pub end: f32,
}

impl SegmentDescription {
    /// Split a single segment description into `n` subsegments,
    /// this is used to assign a solid color to each subsegment and thus
    /// simulate a smooth gradient
    pub fn get_subsegments(&self, num_subsegments: usize) -> impl Iterator<Item = Subsegment> + '_ {
        let SegmentFraction { start, end } = self.fraction;
        let segment_size = self.fraction.end - self.fraction.start;

        let get_color = self.segment_style.color_at_fraction();

        let start_subsegment = (num_subsegments as f32 * start) as usize;
        let end_subsegment = (num_subsegments as f32 * end).ceil() as usize;
        // the actual number of subsegments (partial segments will
        //  have fewer than expected)
        let real_num_subsegments = (start_subsegment..end_subsegment).len();
        let subsegment_size = segment_size / real_num_subsegments as f32;

        // TODO: make sure we're not generating duplicate colors (that num_subsegments isn't too high)
        // the order is tail to head (opposite to the order in which snake segments are rendered)
        (start_subsegment..end_subsegment)
            // TODO: this is very awkward, remove double enumerate
            .rev()
            .enumerate()
            .rev()
            .map(move |(subsegment_idx, subsegment)| {
                (
                    subsegment_idx,
                    get_color(subsegment as f64 / num_subsegments as f64),
                )
            })
            .enumerate()
            .map(move |(i, (subsegment_idx, color))| {
                let end = self.fraction.start + subsegment_size * (i + 1) as f32;
                Subsegment { subsegment_idx, color, end }
            })
    }
}
