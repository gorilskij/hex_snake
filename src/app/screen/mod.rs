pub use debug_scenario::DebugScenario;
pub use game::Game;
pub use start_screen::StartScreen;

pub use crate::app::prefs::Prefs;
use ggez::event::EventHandler;
use crate::app::snake::{Snake, Seed};
use crate::app::apple::Apple;
use crate::basic::{HexDim, FrameStamp};
use rand::Rng;
use rand::rngs::ThreadRng;

mod debug_scenario;
mod game;
mod start_screen;

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
    fn board_dim(&self) -> HexDim;
    fn frame_stamp(&self) -> FrameStamp;
    fn rng(&mut self) -> &mut R;
}
