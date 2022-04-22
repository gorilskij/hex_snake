use ggez::{
    event::{EventHandler, KeyCode, KeyMods},
    graphics,
    graphics::Rect,
    Context,
};
use itertools::Itertools;

use crate::{
    app::{
        app_error::{AppError, AppErrorConversion, AppResult},
        screen::{DebugScenario, StartScreen},
    },
    basic::CellDim,
};
use apple::spawn::SpawnPolicy;
use keyboard_control::ControlSetup;
pub use palette::Palette;
use screen::{Game, Screen};
use snake::{controller, EatBehavior, EatMechanics, Seed};

pub mod keyboard_control;
mod palette;
mod snake;
mod snake_management;
#[macro_use]
mod apple;
mod app_error;
mod control;
mod game_context;
mod message;
mod prefs;
mod rendering;
mod screen;
pub mod stats;
mod utils;

pub struct App {
    screen: Screen,
}

impl App {
    pub fn new(players: Vec<ControlSetup>, ctx: &mut Context) -> Self {
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
                snake_type: snake::Type::Player,
                eat_mechanics: EatMechanics {
                    eat_self: EatBehavior::Cut,
                    eat_other: hash_map! {},
                    default: EatBehavior::Crash,
                },
                palette: snake::PaletteTemplate::rainbow(true),
                // palette: PaletteTemplate::dark_blue_to_red(false),
                // palette: PaletteTemplate::zebra(),
                controller: controller::Template::Keyboard(cs),
                // controller: controller::Template::Mouse,
                // controller: SnakeControllerTemplate::PlayerController12,
                pos: None,
                dir: None,
                len: None,
            })
            .collect();

        let cell_dim = CellDim::from(30.);

        // Manual selection of what to launch
        Self {
            screen: match 0 {
                5 => Screen::DebugScenario(DebugScenario::double_head_body_collision(cell_dim)),
                4 => Screen::DebugScenario(DebugScenario::many_snakes()),
                3 => Screen::DebugScenario(DebugScenario::head_body_collision(cell_dim)),
                2 => Screen::DebugScenario(DebugScenario::head_head_collision(cell_dim)),
                1 => Screen::StartScreen(StartScreen::new(cell_dim)),
                0 => Screen::Game(Game::new(
                    cell_dim,
                    7.,
                    seeds,
                    Palette::dark(),
                    SpawnPolicy::Random { apple_count: 5 },
                    ctx,
                )),
                _ => unreachable!(),
            },
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
        //         SimMove::Move(Dir::Dr),
        //         SimMove::Move(Dir::Dl),
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
}

impl EventHandler<AppError> for App {
    fn update(&mut self, ctx: &mut Context) -> AppResult {
        if let Screen::StartScreen(start_screen) = &self.screen {
            if let Some(next_screen) = start_screen.next_screen() {
                self.screen = next_screen
            }
        }
        self.screen.update(ctx).with_trace_step("App::update")
    }

    fn draw(&mut self, ctx: &mut Context) -> AppResult {
        self.screen.draw(ctx).with_trace_step("App::draw")
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, repeat: bool) {
        self.screen.key_down_event(ctx, key, mods, repeat)
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(ctx, Rect { x: 0.0, y: 0.0, w: width, h: height })
            .unwrap();

        self.screen.resize_event(ctx, width, height);
    }
}
