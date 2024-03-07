// TODO: move this to rendering

use ggez::event::EventHandler;
use ggez::graphics::{Canvas, Rect};
use ggez::input::keyboard::KeyInput;
use ggez::Context;
use itertools::Itertools;
use keyboard_control::ControlSetup;
pub use palette::Palette;
use screen::{Game, Screen};
use snake::builder::Builder as SnakeBuilder;

use crate::app::screen::{DebugScenario, StartScreen};
use crate::apple::spawn::SpawnPolicy;
use crate::basic::CellDim;
use crate::error::{Error, ErrorConversion, Result};
use crate::snake::eat_mechanics::{EatBehavior, EatMechanics, Knowledge};
use crate::snake::SegmentType;
use crate::snake_control::pathfinder;
use crate::{by_segment_type, by_snake_type, snake, snake_control};

mod distance_grid;
pub(crate) mod fps_control;
pub mod game_context;
pub mod keyboard_control;
pub mod message;
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
                let eat_mechanics = EatMechanics::new(
                    by_segment_type! {
                        SegmentType::DISCR_EATEN => EatBehavior::PassOver,
                        _ => EatBehavior::Crash,
                    },
                    by_snake_type! {
                        // TODO: this doesn't work as expected
                        snake::Type::Rain => by_segment_type! {
                            _ => EatBehavior::PassUnder,
                        },
                        _ => by_segment_type! {
                            _ => EatBehavior::Crash,
                        },
                    },
                );

                let knowledge = Knowledge::accurate(&eat_mechanics);

                SnakeBuilder::default()
                    .snake_type(snake::Type::Player)
                    .eat_mechanics(eat_mechanics)
                    .palette(snake::PaletteTemplate::rainbow(true))
                    // .palette(PaletteTemplate::dark_blue_to_red(false))
                    // .palette(PaletteTemplate::zebra())
                    .controller(snake_control::Template::Keyboard { control_setup, knowledge })
                    .speed(1.)
                    .autopilot(pathfinder::Template::WithBackup {
                        main: Box::new(pathfinder::Template::WeightedBFS),
                        backup: Box::new(pathfinder::Template::SpaceFilling),
                    })
                // .snake_control(snake_control::Template::Mouse)
                // .snake_control(SnakeControllerTemplate::PlayerController12)
            })
            .collect();

        let cell_dim = CellDim::from(50.);

        // Manual selection of what to launch
        Self {
            screen: match 0 {
                6 => Screen::DebugScenario(DebugScenario::head_head_collision_apple(cell_dim)),
                5 => Screen::DebugScenario(DebugScenario::double_head_body_collision(cell_dim)),
                // 4 => Screen::DebugScenario(DebugScenario::many_snakes()),
                3 => Screen::DebugScenario(DebugScenario::head_body_collision(cell_dim)),
                2 => Screen::DebugScenario(DebugScenario::head_head_collision(cell_dim)),
                1 => Screen::StartScreen(StartScreen::new(cell_dim, Palette::dark())),
                0 => Screen::Game(Game::new(
                    cell_dim,
                    3.,
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

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) -> Result {
        self.screen.mouse_motion_event(ctx, x, y, dx, dy)
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, repeated: bool) -> Result {
        self.screen.key_down_event(ctx, input, repeated)
    }

    fn key_up_event(&mut self, ctx: &mut Context, input: KeyInput) -> Result {
        self.screen.key_up_event(ctx, input)
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) -> Result {
        Canvas::from_frame(ctx, None).set_screen_coordinates(Rect { x: 0.0, y: 0.0, w: width, h: height });

        self.screen.resize_event(ctx, width, height)
    }
}
