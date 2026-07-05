use crate::basic::{Dir, Point};
use crate::support::mesh::{build_shaded_polygon, build_shaded_ribbon, Mesh};
use crate::rendering;
use crate::rendering::segments::descriptions::SegmentDescription;
use crate::rendering::segments::hexagon_segments::hexagon_outline;
use crate::rendering::segments::smooth_segments::segment_cross_sections;

impl SegmentDescription {
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
        let seg_idx = self.segment_idx as f32;
        let num = num_segments.max(1) as f32;
        // half a texel, in uv units — the shader clamps to these bounds
        let half_texel = 0.5 / lut_size.max(1) as f32;

        match self.draw_style {
            rendering::Style::Hexagon => {
                let points = hexagon_outline(self);
                // one flat color per hexagon: sample the segment's midpoint
                let u = (seg_idx + 0.5) / num;
                build_shaded_polygon(&points, move |_| (u, 0.5), |p| p)
            }
            rendering::Style::Smooth => {
                let (cross_sections, cw) = segment_cross_sections(self);

                let center = self.cell_dim.center();
                let rotation_angle = Dir::U.clockwise_angle_to(self.turn.coming_from);
                let dest = self.destination;

                // Body coordinate runs head→tail. The head-side of a segment is
                // at frac == 1, so its head→tail offset is (1 - frac); the global
                // coordinate is seg_idx + that.
                let u_of = move |frac: f32| (seg_idx + (1.0 - frac)) / num;
                let transform = move |mut p: Point| {
                    // mirrors the old ShapePoints flip/rotate/translate
                    if cw {
                        p.x = 2. * center.x - p.x;
                    }
                    if rotation_angle != 0. {
                        p = p.rotate_clockwise(center, rotation_angle);
                    }
                    p + dest
                };

                // This segment owns uv range [seg_idx/num, (seg_idx+1)/num];
                // pass it inset by half a texel so the shader's clamp keeps
                // linear filtering from bleeding into the neighbor segment.
                let seg_bounds = (seg_idx / num + half_texel, (seg_idx + 1.0) / num - half_texel);
                build_shaded_ribbon(&cross_sections, seg_bounds, u_of, transform)
            }
        }
    }
}
