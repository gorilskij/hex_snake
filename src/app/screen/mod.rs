use anyhow::{Context, Result};
pub use game::Game;
pub use start_screen::StartScreen;
use macroquad::input::KeyCode;
use rand::rngs::ThreadRng;

use crate::app::game_context::GameContext;
use crate::app::key::KeyPress;
use crate::app::portal::Portal;
use crate::apple::Apple;
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::Snake;

mod board_dim;
mod controls_menu;
mod game;
mod menu;
mod options_menu;
mod start_screen;

/// The interface every screen (the game, and eventually menus/settings/editor)
/// implements so the main loop can drive it uniformly: per-frame `update`/`draw`
/// plus input and resize hooks. Screens only override the hooks they use.
#[allow(unused_variables)]
pub trait Screen {
    fn update(&mut self) -> Result<()>;

    fn draw(&mut self) -> Result<()>;

    fn key_down_event(&mut self, press: KeyPress) -> Result<()> {
        Ok(())
    }

    fn key_up_event(&mut self, keycode: KeyCode) -> Result<()> {
        Ok(())
    }

    fn resize_event(&mut self, width: f32, height: f32) -> Result<()> {
        Ok(())
    }

    /// Whether to switch screens (e.g. the start screen once the player
    /// starts a game). Polled once per frame.
    fn transition(&mut self) -> Option<Transition> {
        None
    }

    /// Called when the screen above this one is closed and this one is shown
    /// again.
    fn resume(&mut self) {}
}

/// A change of screen. Screens form a stack: the game is opened over the start
/// screen, and closing it goes back to the start screen as it was left.
pub enum Transition {
    /// Open a screen over this one
    Push(Box<dyn Screen>),
    /// Close this screen, going back to the one below
    Pop,
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
