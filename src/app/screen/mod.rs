pub use game::Game;
use macroquad::input::KeyCode;
use rand::rngs::ThreadRng;

use crate::app::game_context::GameContext;
use crate::app::portal::Portal;
pub use crate::app::prefs::Prefs;
use crate::apple::Apple;
use anyhow::{Context, Result};
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::Snake;

mod board_dim;
mod game;

/// The interface every screen (the game, and eventually menus/settings/editor)
/// implements so the main loop can drive it uniformly: per-frame `update`/`draw`
/// plus input and resize hooks. Screens only override the hooks they use.
#[allow(unused_variables)]
pub trait Screen {
    fn update(&mut self) -> Result<()>;

    fn draw(&mut self) -> Result<()>;

    fn key_down_event(&mut self, keycode: KeyCode) -> Result<()> {
        Ok(())
    }

    fn key_up_event(&mut self, keycode: KeyCode) -> Result<()> {
        Ok(())
    }

    fn resize_event(&mut self, width: f32, height: f32) -> Result<()> {
        Ok(())
    }
}

pub struct Environment<Rng = ThreadRng> {
    pub snakes: Vec<Snake>,
    // TODO: keep apples in order of position to allow for binary search
    // TODO: specialized Vec for that
    pub apples: Vec<Apple>,
    pub portals: Vec<Portal>,
    pub gtx: GameContext,
    pub rng: Rng,
}

impl<Rng> Environment<Rng> {
    pub fn add_snake(&mut self, snake_builder: &SnakeBuilder) -> Result<()> {
        self.snakes
            .push(snake_builder.build().context("Environment::add_snake")?);
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
