pub use debug_scenario::DebugScenario;
pub use game::Game;
pub use start_screen::StartScreen;

pub use crate::app::prefs::Prefs;
use crate::{
    app::{
        apple::Apple,
        snake::{Seed, Snake},
    },
    basic::{FrameStamp, HexDim},
};

use rand::{rngs::ThreadRng, Rng};
use crate::basic::CellDim;
use crate::app::game_context::GameContext;

mod debug_scenario;
mod game;
mod start_screen;
mod board_dim;

#[allow(dead_code)]
pub enum Screen {
    DebugScenario(DebugScenario),
    StartScreen(StartScreen),
    Game(Game),
}

pub trait Environment<R: Rng = ThreadRng> {
    fn snakes(&self) -> &[Snake];
    fn apples(&self) -> &[Apple];
    fn snakes_apples_gtx_mut(&mut self) -> (&mut [Snake], &mut [Apple], &mut GameContext);
    fn add_snake(&mut self, seed: &Seed);
    fn remove_snake(&mut self, index: usize) -> Snake;
    fn remove_apple(&mut self, index: usize) -> Apple;
    fn gtx(&self) -> &GameContext;
    // TODO: remove redundant methods
    fn board_dim(&self) -> HexDim { self.gtx().board_dim }
    fn cell_dim(&self) -> CellDim { self.gtx().cell_dim }
    fn frame_stamp(&self) -> FrameStamp { self.gtx().frame_stamp }
    fn rng(&mut self) -> &mut R;
}
