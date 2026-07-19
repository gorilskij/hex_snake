use crate::basic::Point;
use crate::rendering::segments::descriptions::SegmentDescription;
use crate::rendering::shape::{Hexagon, Shape};

/// Hexagon outline for a segment, positioned on the board. The hexagon draw
/// style makes no distinction between straight, blunt, and sharp segments — it
/// just draws a full hexagon per cell (colored flat via the shader).
pub fn hexagon_outline(description: &SegmentDescription) -> Vec<Point> {
    Hexagon::new(description.cell_dim)
        .translate(description.destination)
        .into()
}
