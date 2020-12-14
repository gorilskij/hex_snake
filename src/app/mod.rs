use ggez::event::EventHandler;
use crate::app::start_screen::StartScreen;
use ggez::conf::{WindowMode, WindowSetup, FullscreenType, NumSamples};
use ggez::{Context, GameResult};
use crate::app::game::Game;
use std::ops::{DerefMut, Deref};

mod start_screen;
mod game;
mod hex;
mod snake;
#[macro_use]
#[allow(dead_code)]
pub mod ctrl;
mod palette;

pub enum Screen {
    StartScreen(StartScreen),
    Game(Game),
}

impl Deref for Screen {
    type Target = dyn EventHandler;

    fn deref(&self) -> &Self::Target {
        use Screen::*;
        match self {
            StartScreen(x) => x,
            Game(x) => x,
        }
    }
}

impl DerefMut for Screen {
    fn deref_mut(&mut self) -> &mut Self::Target {
        use Screen::*;
        match self {
            StartScreen(x) => x,
            Game(x) => x,
        }
    }
}

pub struct App {
    screen: Screen,
    wm: WindowMode,
    ws: WindowSetup,
}

impl App {
    pub fn new() -> Self {
        let wm = WindowMode {
            width: 1000.,
            height: 800.,
            maximized: false,
            fullscreen_type: FullscreenType::Windowed,
            borderless: false,
            min_width: 0.,
            min_height: 0.,
            max_width: 0.,
            max_height: 0.,
            // resizable: false,
            resizable: true,
        };

        let ws = WindowSetup {
            title: "Hex Snake".to_string(),
            samples: NumSamples::Zero,
            vsync: true,
            icon: "".to_string(),
            srgb: true,
        };

        Self {
            screen: Screen::StartScreen(StartScreen::new()),
            // screen: Box::new(Game::new()),
            wm,
            ws
        }
    }

    pub fn wm(&self) -> WindowMode {
        self.wm.clone()
    }

    pub fn ws(&self) -> WindowSetup {
        self.ws.clone()
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if let Screen::StartScreen(start_screen) = &self.screen {
            if let Some(next_screen) = start_screen.next_screen() {
                self.screen = next_screen
            }
        }
        self.screen.update(ctx)
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.screen.draw(ctx)
    }
}
