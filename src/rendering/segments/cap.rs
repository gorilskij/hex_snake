//! Round end caps for smooth-style snakes.
//!
//! Each end of the snake is capped with a half-circle of radius half the
//! body's width, whose apex sits exactly where the flat end face used to be:
//! the body ribbon is truncated by one cap radius of *path length* at each end
//! and the cap fills the removed range with cross-sections of shrinking width.
//!
//! Because truncation is measured along the body's path, a cap near a cell
//! boundary naturally straddles it: the part behind the boundary is generated
//! from the previous segment's geometry (bending with its arc), the part ahead
//! from the end segment's, and the two ribbons share the boundary
//! cross-section exactly like body segments do — so caps slide seamlessly
//! across cell boundaries and bend around turns.

use crate::basic::Point;
use crate::rendering::segments::descriptions::SegmentDescription;
use crate::rendering::segments::smooth_segments::{cross_section_at, full_path_length};
use crate::snake::SegmentType;
use crate::support::mesh::{build_shaded_ribbon, Mesh};

/// Target on-screen spacing (px) between successive cap cross-sections.
const CAP_STEP: f32 = 3.0;

#[derive(Copy, Clone)]
enum End {
    Head,
    Tail,
}

impl End {
    /// Which fraction of a segment faces the cap's tip.
    fn tip_frac(self, desc: &SegmentDescription) -> f32 {
        match self {
            End::Head => desc.fraction.end,
            End::Tail => desc.fraction.start,
        }
    }

    /// Direction of increasing path distance from the tip, in fraction terms.
    fn away_sign(self) -> f32 {
        match self {
            End::Head => -1.,
            End::Tail => 1.,
        }
    }
}

/// A contiguous piece of one cap inside a single segment's cell.
struct CapSpan {
    desc_idx: usize,
    /// Segment fraction at the tip side of the span.
    tip_frac: f32,
    /// Path distance from the cap tip to the tip side of the span.
    tip_dist: f32,
    /// Path length covered by the span.
    len: f32,
}

/// Build round caps for both ends of the snake and truncate the body ribbon
/// underneath them. `descs` is ordered head→tail; the segments' drawn
/// fractions are shortened in place by one cap radius at each capped end.
/// Returns `(tail_cap, head_cap)` so the caller can keep the tail-under,
/// head-over draw order.
pub fn build_round_caps(
    descs: &mut [SegmentDescription],
    num_segments: usize,
    lut_size: usize,
) -> (Option<Mesh>, Option<Mesh>) {
    let half_width = descs[0].cell_dim.side / 2.;

    let lens: Vec<f32> = descs.iter().map(full_path_length).collect();
    let total: f32 = descs
        .iter()
        .zip(&lens)
        .map(|(desc, len)| (desc.fraction.end - desc.fraction.start).max(0.) * len)
        .sum();

    // a head crashed into an obstacle keeps its flat face
    let round_head = !matches!(descs[0].segment_type, SegmentType::Crashed);

    // Never consume more than the whole body: while the snake is very short
    // (e.g. just spawning) the caps shrink to half-ellipses that meet in the
    // middle.
    let n_caps = 1 + round_head as usize;
    let radius = half_width.min(total / n_caps as f32);
    if radius <= f32::EPSILON {
        return (None, None);
    }

    // collect both caps' spans before truncating anything
    let head_spans = round_head.then(|| collect_spans(descs, &lens, End::Head, radius));
    let tail_spans = collect_spans(descs, &lens, End::Tail, radius);

    let head_cap = head_spans.map(|spans| {
        let mesh = build_cap(descs, &lens, &spans, End::Head, radius, num_segments, lut_size);
        truncate(descs, &lens, &spans, End::Head);
        mesh
    });
    let tail_cap = {
        let mesh = build_cap(descs, &lens, &tail_spans, End::Tail, radius, num_segments, lut_size);
        truncate(descs, &lens, &tail_spans, End::Tail);
        Some(mesh)
    };

    (tail_cap, head_cap)
}

/// Walk segments away from the given end, splitting the first `radius` of
/// path length into per-segment spans (usually one span; two when the cap
/// straddles a cell boundary).
fn collect_spans(descs: &[SegmentDescription], lens: &[f32], end: End, radius: f32) -> Vec<CapSpan> {
    let indices: Box<dyn Iterator<Item = usize>> = match end {
        End::Head => Box::new(0..descs.len()),
        End::Tail => Box::new((0..descs.len()).rev()),
    };

    let mut spans = vec![];
    let mut consumed = 0.;
    for idx in indices {
        if consumed >= radius - f32::EPSILON {
            break;
        }
        let desc = &descs[idx];
        let extent = (desc.fraction.end - desc.fraction.start).max(0.) * lens[idx];
        if extent <= f32::EPSILON {
            continue;
        }
        let take = extent.min(radius - consumed);
        spans.push(CapSpan {
            desc_idx: idx,
            tip_frac: end.tip_frac(desc),
            tip_dist: consumed,
            len: take,
        });
        consumed += take;
    }
    spans
}

/// Shorten the body ribbon by the path length covered by the cap spans.
fn truncate(descs: &mut [SegmentDescription], lens: &[f32], spans: &[CapSpan], end: End) {
    for span in spans {
        let desc = &mut descs[span.desc_idx];
        let d_frac = span.len / lens[span.desc_idx];
        match end {
            End::Head => desc.fraction.end -= d_frac,
            End::Tail => desc.fraction.start += d_frac,
        }
    }
}

/// Build one cap as a ribbon of width-scaled cross-sections, one sub-ribbon
/// per span. Adjacent spans both emit the shared boundary cross-section, so
/// the cap is continuous across cell boundaries.
///
/// The cap profile is parameterized by `phi ∈ [0, π/2]`: path distance from
/// the cap base is `radius·sin(phi)` and the lateral half-width is the body's
/// half-width times `cos(phi)` — a half-circle when `radius` equals the
/// half-width, a half-ellipse when the cap had to shrink.
#[allow(clippy::too_many_arguments)]
fn build_cap(
    descs: &[SegmentDescription],
    lens: &[f32],
    spans: &[CapSpan],
    end: End,
    radius: f32,
    num_segments: usize,
    lut_size: usize,
) -> Mesh {
    let phi_of = |dist_from_tip: f32| ((1. - dist_from_tip / radius).clamp(-1., 1.)).asin();

    // spans are ordered tip→base; emit the ribbon base→tip
    let parts = spans.iter().rev().map(|span| {
        let desc = &descs[span.desc_idx];

        let phi_tip_side = phi_of(span.tip_dist);
        let phi_base_side = phi_of(span.tip_dist + span.len);
        let steps = ((radius * (phi_tip_side - phi_base_side) / CAP_STEP).ceil() as usize).clamp(2, 32);

        let sections: Vec<(Point, Point, f32)> = (0..=steps)
            .map(|k| {
                let t = k as f32 / steps as f32;
                let phi = phi_base_side + (phi_tip_side - phi_base_side) * t;
                let dist_from_tip = radius * (1. - phi.sin());
                let frac = span.tip_frac + end.away_sign() * (dist_from_tip - span.tip_dist) / lens[span.desc_idx];

                // full-width cross-section, scaled towards its midpoint
                let (inner, outer) = cross_section_at(desc, frac);
                let mid = (inner + outer) * 0.5;
                let scale = phi.cos();
                (mid + (inner - mid) * scale, mid + (outer - mid) * scale, frac)
            })
            .collect();

        build_shaded_ribbon(
            &sections,
            desc.seg_bounds(num_segments, lut_size),
            desc.u_of(num_segments),
            desc.board_transform(),
        )
    });

    Mesh::combine(parts)
}
