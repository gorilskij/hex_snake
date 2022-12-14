use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Rect};
use ggez::Context;
use itertools::Itertools;

use crate::app::guidance::PathFinderTemplate;
use crate::app::screen::{DebugScenario, StartScreen};
use crate::apple::spawn::SpawnPolicy;
use crate::basic::CellDim;
use crate::error::{Error, ErrorConversion, Result};
use crate::snake::{
    EatBehavior, EatMechanics, PassthroughKnowledge, SegmentRawType, {self},
};
use crate::snake_control;
use keyboard_control::ControlSetup;
pub use palette::Palette;
use screen::{Game, Screen};

mod distance_grid;
mod fps_control;
pub mod game_context;
pub mod guidance;
pub mod keyboard_control;
mod message;
mod palette;
mod prefs;
pub(crate) mod screen;
mod snake_management;
pub mod stats;

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
            .map(|control_setup| {
                let eat_mechanics = EatMechanics {
                    eat_self: hash_map_with_default! {
                        default => EatBehavior::Crash,
                        SegmentRawType::Eaten => EatBehavior::PassUnder,
                    },
                    eat_other: hash_map_with_default! {
                        default => hash_map_with_default! {
                            default => EatBehavior::Crash,
                        },
                        snake::Type::Rain => hash_map_with_default! {
                            default => EatBehavior::PassUnder,
                        },
                    },
                };

                let passthrough_knowledge = PassthroughKnowledge::accurate(&eat_mechanics);

                snake::Builder::default()
                    .snake_type(snake::Type::Player)
                    .eat_mechanics(eat_mechanics)
                    .palette(snake::PaletteTemplate::rainbow(true))
                    // .palette(PaletteTemplate::dark_blue_to_red(false))
                    // .palette(PaletteTemplate::zebra())
                    .controller(snake_control::Template::Keyboard {
                        control_setup,
                        passthrough_knowledge,
                    })
                    .speed(1.)
                    .autopilot(PathFinderTemplate::Algorithm1)
                // .snake_control(snake_control::Template::Mouse)
                // .snake_control(SnakeControllerTemplate::PlayerController12)
            })
            .collect();

        let cell_dim = CellDim::from(30.);

        // Manual selection of what to launch
        Self {
            screen: match 0 {
                5 => Screen::DebugScenario(DebugScenario::double_head_body_collision(cell_dim)),
                // 4 => Screen::DebugScenario(DebugScenario::many_snakes()),
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
        //     snake_control: SnakeControllerTemplate::demo_triangle_pattern(0, Side::Right),
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
        //     snake_control: SnakeControllerTemplate::demo_hexagon_pattern(0),
        // }];

        // let seeds = vec![SnakeSeed {
        //     snake_type: SnakeType::PlayerSnake,
        //     eat_mechanics: EatMechanics::always(EatBehavior::Cut),
        //     palette: SnakePaletteTemplate::rainbow(),
        //     snake_control: SnakeControllerTemplate::DemoController(vec![
        //         SimMove::Move(Dir::U),
        //         SimMove::Move(Dir::Dr),
        //         SimMove::Move(Dir::Dl),
        //     ]),
        // }];

        // let seeds = vec![SnakeSeed {
        //     snake_type: SnakeType::PlayerSnake,
        //     eat_mechanics: EatMechanics::always(EatBehavior::Cut),
        //     palette: SnakePaletteTemplate::gray_gradient(),
        //     snake_control: SnakeControllerTemplate::CompetitorAI2,
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

impl EventHandler<Error> for App {
    fn update(&mut self, ctx: &mut Context) -> Result {
        if let Screen::StartScreen(start_screen) = &self.screen {
            if let Some(next_screen) = start_screen.next_screen() {
                self.screen = next_screen
            }
        }
        self.screen.update(ctx).with_trace_step("App::update")
    }

    fn draw(&mut self, ctx: &mut Context) -> Result {
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
