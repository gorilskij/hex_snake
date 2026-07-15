use crate::basic::{Dir, Point};
use crate::rendering;
use crate::rendering::segments::descriptions::{SegmentDescription, TurnDirection, TurnType};
use crate::rendering::segments::hexagon_segments::hexagon_outline;
use crate::rendering::segments::smooth_segments::segment_cross_sections;
use crate::support::mesh::{build_shaded_polygon, build_shaded_ribbon, Mesh};

impl SegmentDescription {
    /// Whether this segment's default-orientation geometry must be mirrored
    /// horizontally when placed on the board (arcs are generated
    /// counterclockwise; clockwise turns are their mirror image). Mirroring a
    /// straight box is harmless (it is symmetric about the cell axis).
    pub fn is_flipped(&self) -> bool {
        matches!(
            self.turn.turn_type(),
            TurnType::Blunt(TurnDirection::Clockwise) | TurnType::Sharp(TurnDirection::Clockwise)
        )
    }

    /// The flip/rotate/translate that places default-orientation points on the
    /// board (mirrors the old `ShapePoints` transform chain).
    pub fn board_transform(&self) -> impl Fn(Point) -> Point {
        let flip = self.is_flipped();
        let center = self.cell_dim.center();
        let rotation_angle = Dir::U.clockwise_angle_to(self.turn.coming_from);
        let dest = self.destination;

        move |mut p: Point| {
            if flip {
                p.x = 2. * center.x - p.x;
            }
            if rotation_angle != 0. {
                p = p.rotate_clockwise(center, rotation_angle);
            }
            p + dest
        }
    }

    /// Maps a segment-local fraction to the global body coordinate `uv.x`.
    /// Body coordinate runs head→tail. The head-side of a segment is at
    /// frac == 1, so its head→tail offset is (1 - frac); the global coordinate
    /// is seg_idx + that.
    pub fn u_of(&self, num_segments: usize) -> impl Fn(f32) -> f32 {
        let seg_idx = self.segment_idx as f32;
        let num = num_segments.max(1) as f32;
        move |frac: f32| (seg_idx + (1.0 - frac)) / num
    }

    /// This segment's `(lo, hi)` uv range in the palette LUT, inset by half a
    /// texel so the shader's clamp keeps linear filtering from bleeding into
    /// the neighbor segment.
    pub fn seg_bounds(&self, num_segments: usize, lut_size: usize) -> (f32, f32) {
        let seg_idx = self.segment_idx as f32;
        let num = num_segments.max(1) as f32;
        // half a texel, in uv units
        let half_texel = 0.5 / lut_size.max(1) as f32;
        (seg_idx / num + half_texel, (seg_idx + 1.0) / num - half_texel)
    }

    /// Tessellate this segment into a single shaded [`Mesh`]. Each vertex carries
    /// `uv.x` = position along the whole body (so the snake shader can sample the
    /// palette LUT) and `uv.y` = across width. `num_segments` is the snake's
    /// segment count (normalizes the body coordinate into `[0, 1]`); `lut_size`
    /// is the LUT texture width (used to inset this segment's clamp bounds by
    /// half a texel).
    ///
    /// Replaces the old subsegment approach: instead of many flat-colored
    /// slices, the segment is one polygon and color is per-pixel on the GPU.
    pub fn build_shaded(&self, num_segments: usize, lut_size: usize) -> Mesh {
        match self.draw_style {
            rendering::Style::Hexagon => {
                let seg_idx = self.segment_idx as f32;
                let num = num_segments.max(1) as f32;
                let points = hexagon_outline(self);
                // one flat color per hexagon: sample the segment's midpoint
                let u = (seg_idx + 0.5) / num;
                build_shaded_polygon(&points, move |_| (u, 0.5), |p| p)
            }
            rendering::Style::Smooth => {
                let (cross_sections, _cw) = segment_cross_sections(self);
                build_shaded_ribbon(
                    &cross_sections,
                    self.seg_bounds(num_segments, lut_size),
                    self.u_of(num_segments),
                    self.board_transform(),
                )
            }
        }
    }
}
