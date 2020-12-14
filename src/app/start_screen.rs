use ggez::event::EventHandler;
use ggez::{Context, GameResult};
use super::palette::SnakePalette;
use ggez::graphics::{clear, present, BLACK};
use crate::app::Screen;

pub struct StartScreen {
    // options selected
    players: usize,
    player1_palette: SnakePalette,
    player2_palette: Option<SnakePalette>,
}

impl StartScreen {
    pub fn new() -> Self {
        Self {
            players: 1,
            player1_palette: SnakePalette::rainbow(),
            player2_palette: None,
        }
    }
}

impl EventHandler for StartScreen {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        clear(ctx, BLACK);



        present(ctx)
    }
}

impl StartScreen {
    pub fn next_screen(&self) -> Option<Screen> {
        None
    }
}