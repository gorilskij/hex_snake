use crate::app;
use crate::app::prefs::Prefs;
use crate::apple::spawn::SpawnPolicy;
use crate::basic::{CellDim, HexDim};

// TODO: add Stats to game context
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
}

impl GameContext {
    pub fn new(
        board_dim: HexDim,
        cell_dim: CellDim,
        palette: app::Palette,
        prefs: Prefs,
        apple_spawn_policy: SpawnPolicy
    ) -> Self {
        Self {
            board_dim,
            cell_dim,
            palette,
            prefs,
            apple_spawn_policy,
        }
    }
}
