pub use debug_scenario::DebugScenario;
pub use game::Game;
pub use start_screen::StartScreen;

pub use crate::app::prefs::Prefs;
use crate::basic::{FrameStamp, HexDim};
use crate::snake;

use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::CellDim;
use crate::error::{Error, Result};
use crate::snake::Snake;
use ggez::event::EventHandler;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::ops::{Deref, DerefMut};

mod board_dim;
mod debug_scenario;
mod game;
mod start_screen;

#[allow(dead_code)]
pub enum Screen {
    DebugScenario(DebugScenario),
    StartScreen(StartScreen),
    Game(Game),
}

impl Deref for Screen {
    type Target = dyn EventHandler<Error>;

    fn deref(&self) -> &Self::Target {
        use Screen::*;
        match self {
            DebugScenario(x) => x,
            StartScreen(x) => x,
            Game(x) => x,
        }
    }
}

impl DerefMut for Screen {
    fn deref_mut(&mut self) -> &mut Self::Target {
        use Screen::*;
        match self {
            DebugScenario(x) => x,
            StartScreen(x) => x,
            Game(x) => x,
        }
    }
}

// TODO: refactor into view
pub trait Environment<R: Rng = ThreadRng> {
    fn snakes(&self) -> &[Snake];
    fn apples(&self) -> &[Apple];
    fn snakes_apples_gtx_mut(&mut self) -> (&mut [Snake], &mut [Apple], &mut GameContext);
    fn snakes_apples_rng_mut(&mut self) -> (&mut [Snake], &mut [Apple], &mut R);
    fn add_snake(&mut self, snake_builder: &snake::Builder) -> Result;
    fn remove_snake(&mut self, index: usize) -> Snake;
    fn remove_apple(&mut self, index: usize) -> Apple;
    fn gtx(&self) -> &GameContext;
    // TODO: remove redundant methods
    fn board_dim(&self) -> HexDim {
        self.gtx().board_dim
    }
    fn cell_dim(&self) -> CellDim {
        self.gtx().cell_dim
    }
    fn frame_stamp(&self) -> FrameStamp {
        self.gtx().frame_stamp
    }
    fn rng(&mut self) -> &mut R;
}
