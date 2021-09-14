use std::ops::{Deref, DerefMut};

use ggez::{
    conf::{WindowMode, WindowSetup},
    Context,
    event::{EventHandler, KeyCode, KeyMods},
    GameResult, graphics,
    graphics::Rect,
};
use itertools::Itertools;

use apple::spawn::SpawnPolicy;
pub use palette::Palette;
use screen::{Game, StartScreen};
use snake::{EatMechanics, Seed};

use crate::{
    app::{
        keyboard_control::ControlSetup,
        snake::{controller::ControllerTemplate, EatBehavior, Type},
    },
    basic::CellDim,
};
use crate::app::screen::{DebugScenario, Screen};

pub mod keyboard_control;
mod palette;
mod snake;
mod collisions;
#[macro_use]
mod apple;
mod screen;
mod utils;
mod prefs;
pub mod stats;
mod message;
pub mod control;
mod rendering;

impl Deref for Screen {
    type Target = dyn EventHandler<ggez::GameError>;

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

pub struct App {
    screen: Screen,
    window_mode: WindowMode,
    window_setup: WindowSetup,
}

impl App {
    pub fn new(
        players: Vec<ControlSetup>,
        window_mode: WindowMode,
        window_setup: WindowSetup,
    ) -> Self {
        assert_eq!(
            players.iter().map(|cs| cs.layout).dedup().count(),
            1,
            "found different keyboard layouts for different players"
        );

        assert_eq!(
            players.iter().map(|cs| cs.keyboard_side).dedup().count(),
            players.len(),
            "found multiple players on the same side of the keyboard"
        );

        let seeds: Vec<_> = players
            .into_iter()
            .map(|cs| Seed {
                snake_type: Type::Player,
                eat_mechanics: EatMechanics {
                    eat_self: EatBehavior::Cut,
                    eat_other: hash_map! {},
                    default: EatBehavior::Crash,
                },
                palette: snake::PaletteTemplate::rainbow(true),
                // palette: PaletteTemplate::dark_blue_to_red(false),
                // palette: PaletteTemplate::zebra(),
                controller: ControllerTemplate::Keyboard(cs),
                // controller: SnakeControllerTemplate::PlayerController12,
            })
            .collect();

        let cell_dim = CellDim::from(30.);
        Self {
            // screen: Screen::DebugScenario(DebugScenario::collision1(cell_dim)),
            // screen: Screen::StartScreen(StartScreen::new(cell_dim)),
            screen: Screen::Game(Game::new(
                cell_dim,
                7.,
                seeds,
                Palette::dark(),
                SpawnPolicy::Random { apple_count: 5 },
                window_mode,
            )),
            window_mode,
            window_setup,
        }

        // let seeds = vec![SnakeSeed {
        //     snake_type: SnakeType::SimulatedSnake {
        //         start_pos: HexPoint { h: 10, v: 10 },
        //         start_dir: Dir::U,
        //         start_grow: 2,
        //     },
        //     eat_mechanics: EatMechanics::always(EatBehavior::Cut),
        //     palette: SnakePaletteTemplate::rainbow(),
        //     controller: SnakeControllerTemplate::demo_triangle_pattern(0, Side::Right),
        // }];
        //
        // Self {
        //     // screen: Screen::StartScreen(StartScreen::new()),
        //     screen: Screen::Game(Game::new(
        //         12.,
        //         seeds,
        //         GamePalette::dark(),
        //         AppleSpawnPolicy::ScheduledOnEat {
        //             apple_count: 1,
        //             spawns: spawn_schedule![spawn(10, 9), wait(10),],
        //             next_index: 0,
        //         },
        //         window_mode,
        //     )),
        //     window_mode,
        //     window_setup,
        // }

        // let seeds = vec![SnakeSeed {
        //     snake_type: SnakeType::PlayerSnake,
        //     eat_mechanics: EatMechanics::always(EatBehavior::Cut),
        //     palette: SnakePaletteTemplate::rainbow(),
        //     controller: SnakeControllerTemplate::demo_hexagon_pattern(0),
        // }];

        // let seeds = vec![SnakeSeed {
        //     snake_type: SnakeType::PlayerSnake,
        //     eat_mechanics: EatMechanics::always(EatBehavior::Cut),
        //     palette: SnakePaletteTemplate::rainbow(),
        //     controller: SnakeControllerTemplate::DemoController(vec![
        //         SimMove::Move(Dir::U),
        //         SimMove::Move(Dir::DR),
        //         SimMove::Move(Dir::DL),
        //     ]),
        // }];

        // let seeds = vec![SnakeSeed {
        //     snake_type: SnakeType::PlayerSnake,
        //     eat_mechanics: EatMechanics::always(EatBehavior::Cut),
        //     palette: SnakePaletteTemplate::gray_gradient(),
        //     controller: SnakeControllerTemplate::CompetitorAI2,
        // }];
        //
        // Self {
        //     // screen: Screen::StartScreen(StartScreen::new()),
        //     screen: Screen::Game(Game::new(
        //         12.,
        //         seeds,
        //         GamePalette::dark(),
        //         AppleSpawnPolicy::Random { apple_count: 1 },
        //         window_mode,
        //     )),
        //     window_mode,
        //     window_setup,
        // }
    }

    pub fn wm(&self) -> WindowMode {
        self.window_mode
    }

    pub fn ws(&self) -> WindowSetup {
        self.window_setup.clone()
    }
}

impl EventHandler<ggez::GameError> for App {
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
        println!("pressed {:?}", key);
        self.screen.key_down_event(ctx, key, mods, repeat)
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(ctx, Rect { x: 0.0, y: 0.0, w: width, h: height })
            .unwrap();

        self.screen.resize_event(ctx, width, height);
    }
}
