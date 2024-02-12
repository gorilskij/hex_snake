pub use debug_scenario::DebugScenario;
pub use game::Game;
pub use start_screen::StartScreen;

pub use crate::app::prefs::Prefs;

use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::error::{Error, ErrorConversion, Result};
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::Snake;
use ggez::event::EventHandler;
use rand::rngs::ThreadRng;
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

pub struct Environment<Rng = ThreadRng> {
    pub snakes: Vec<Snake>,
    // TODO: keep apples in order of position to allow for binary search
    // TODO: specialized Vec for that
    pub apples: Vec<Apple>,
    pub gtx: GameContext,
    pub rng: Rng,
}

impl<Rng> Environment<Rng> {
    pub fn add_snake(&mut self, snake_builder: &SnakeBuilder) -> Result {
        self.snakes.push(
            snake_builder
                .build()
                .map_err(Error::from)
                .with_trace_step("Environment::add_snake")?,
        );
        // TODO: check that the snake can be added, report error if it can't
        Ok(())
    }

    pub fn remove_snake(&mut self, index: usize) -> Snake {
        self.snakes.remove(index)
    }

    pub fn remove_apples(&mut self, mut indices: Vec<usize>) {
        indices.sort_unstable();
        indices.dedup();

        indices.into_iter().rev().for_each(|i| {
            self.apples.remove(i);
        });
    }
}
