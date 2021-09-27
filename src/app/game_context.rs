use crate::basic::{HexDim, CellDim, FrameStamp};
use crate::app;
use crate::app::prefs::Prefs;
use crate::app::apple::spawn::SpawnPolicy;

pub struct GameContext {
    pub board_dim: HexDim,
    pub cell_dim: CellDim,
    pub palette: app::Palette,
    pub prefs: Prefs,
    pub apple_spawn_policy: SpawnPolicy,
    pub frame_stamp: FrameStamp,
}
