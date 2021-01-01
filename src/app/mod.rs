use std::ops::{Deref, DerefMut};

use ggez::{Context, GameResult};
use ggez::conf::{FullscreenType, NumSamples, WindowMode, WindowSetup};
use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::graphics::Rect;

use control::{ControlSetup, KeyboardLayout, Side};
use game::Game;
use palette::GamePalette;
use start_screen::StartScreen;

mod start_screen;
mod game;
mod hex;
mod snake;
pub mod control;
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
            // screen: Screen::StartScreen(StartScreen::new()),
            screen: Screen::Game(Game::new(10., vec![
                ControlSetup {
                    layout: KeyboardLayout::Dvorak,
                    keyboard_side: Side::RightSide,
                    hand: Side::RightSide,
                },
                // ControlSetup {
                //     layout: KeyboardLayout::Dvorak,
                //     keyboard_side: Side::LeftSide,
                //     hand: Side::RightSide,
                // },
            ], GamePalette::dark(), wm.clone())),
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
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if let Screen::StartScreen(start_screen) = &self.screen {
            if let Some(next_screen) = start_screen.next_screen() {
                self.screen = next_screen
            }
        }
        self.screen.update(ctx)
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.screen.draw(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, repeat: bool) {
        self.screen.key_down_event(ctx, key, mods, repeat)
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        ggez::graphics::set_screen_coordinates(ctx, Rect {
            x: 0.0,
            y: 0.0,
            w: width,
            h: height,
        }).unwrap();

        self.screen.resize_event(ctx, width, height);
    }
}
