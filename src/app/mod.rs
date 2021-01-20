use std::ops::{Deref, DerefMut};

use ggez::{
    conf::{FullscreenType, NumSamples, WindowMode, WindowSetup},
    event::{EventHandler, KeyCode, KeyMods},
    graphics::Rect,
    Context, GameResult,
};

use crate::app::{
    keyboard_control::{ControlSetup, KeyboardLayout, Side},
    snake::{
        controller::SnakeControllerTemplate, palette::SnakePaletteTemplate, EatBehavior, SnakeType,
    },
};
use game::Game;
use palette::GamePalette;
use snake::{EatMechanics, SnakeSeed};
use start_screen::StartScreen;

macro_rules! hash_map {
    {} => {
        ::std::collections::HashMap::new()
    };
    { $($key:expr => $value:expr),+ } => {{
        let mut map = ::std::collections::HashMap::new();
        $( m.insert($key, $value); )+
        map
    }};
}

mod game;
mod hex;
pub mod keyboard_control;
mod palette;
mod snake;
mod start_screen;

pub type Frames = u32;

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
    window_mode: WindowMode,
    window_setup: WindowSetup,
}

impl App {
    pub fn new() -> Self {
        let window_mode = WindowMode {
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

        let window_setup = WindowSetup {
            title: "Hex Snake".to_string(),
            samples: NumSamples::Zero,
            vsync: true,
            icon: "".to_string(),
            srgb: true,
        };

        Self {
            // screen: Screen::StartScreen(StartScreen::new()),
            screen: Screen::Game(Game::new(
                10.,
                vec![
                    SnakeSeed {
                        snake_type: SnakeType::PlayerSnake,
                        eat_mechanics: EatMechanics {
                            eat_self: EatBehavior::Cut,
                            eat_other: hash_map! {},
                            default: EatBehavior::Crash,
                        },
                        palette: SnakePaletteTemplate::new_persistent_rainbow(),
                        controller: SnakeControllerTemplate::PlayerController(ControlSetup {
                            layout: KeyboardLayout::Dvorak,
                            keyboard_side: Side::Right,
                            hand: Side::Right,
                        }),
                    },
                    // SnakeSeed {
                    //     snake_type: SnakeType::CompetitorSnake { life: None },
                    //     eat_mechanics: EatMechanics {
                    //         eat_self: EatBehavior::Cut,
                    //         eat_other: hash_map! {},
                    //         default: EatBehavior::Cut,
                    //     },
                    //     palette: SnakePaletteTemplate::new_persistent_pastel_rainbow(),
                    //     controller: SnakeControllerTemplate::CompetitorAI,
                    // }; 20
                ],
                GamePalette::dark(),
                window_mode,
            )),
            window_mode,
            window_setup,
        }
    }

    pub fn wm(&self) -> WindowMode {
        self.window_mode
    }

    pub fn ws(&self) -> WindowSetup {
        self.window_setup.clone()
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
        ggez::graphics::set_screen_coordinates(
            ctx,
            Rect {
                x: 0.0,
                y: 0.0,
                w: width,
                h: height,
            },
        )
        .unwrap();

        self.screen.resize_event(ctx, width, height);
    }
}
