use crate::basic::{HexDim, HexPoint};

pub fn a_star(
    pos: HexPoint,
    board_dim: HexDim,
    off_limits: impl IntoIterator<Item = HexPoint>,

    step_cost: f64,
    smooth_turn_cost: f64,
    sharp_turn_cost: f64,
    teleport_cost: f64,
) -> (Vec<HexPoint>, f64) {
    panic!()
}
