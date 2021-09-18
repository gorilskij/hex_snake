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

mod debug_scenario;
mod game;
mod start_screen;

#[allow(dead_code)]
pub enum Screen {
    DebugScenario(DebugScenario),
    StartScreen(StartScreen),
    Game(Game),
}

pub trait Environment<R: Rng = ThreadRng> {
    fn snakes(&self) -> &[Snake];
    fn apples(&self) -> &[Apple];
    fn snakes_apples_mut(&mut self) -> (&mut [Snake], &mut [Apple]);
    fn add_snake(&mut self, seed: &Seed);
    fn remove_snake(&mut self, index: usize) -> Snake;
    fn remove_apple(&mut self, index: usize) -> Apple;
    fn board_dim(&self) -> HexDim;
    fn frame_stamp(&self) -> FrameStamp;
    fn rng(&mut self) -> &mut R;
}
