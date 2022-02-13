use crate::basic::{HexDim, CellDim, FrameStamp};
use crate::app;
use crate::app::prefs::Prefs;
use crate::app::apple::spawn::SpawnPolicy;

pub struct GameContext {
    /// Dimension of the game board in hexagons
    pub board_dim: HexDim,
    /// Graphical dimensions of each hexagonal cell
    pub cell_dim: CellDim,
    /// Color preferences for the game board
    pub palette: app::Palette,
    /// Gameplay preferences
    pub prefs: Prefs,
    /// How many apples are spawned and when
    pub apple_spawn_policy: SpawnPolicy,
    /// Current graphics frame number and frame fraction,
    /// note that the speed of graphics frames is decided
    /// by the ggez runtime
    pub frame_stamp: FrameStamp,
    /// Total number of milliseconds that have elapsed
    /// since the game was started
    pub elapsed_millis: u128,
}
