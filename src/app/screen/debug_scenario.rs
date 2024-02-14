use std::{iter, result};

use ggez::event::EventHandler;
use ggez::graphics::{Canvas, DrawParam};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::Context;
use rand::prelude::*;

use crate::app::fps_control::{self, FpsControl};
use crate::app::game_context::GameContext;
use crate::app::prefs::Prefs;
use crate::app::screen::board_dim::{calculate_board_dim, calculate_offset};
use crate::app::screen::Environment;
use crate::app::snake_management::{advance_snakes, find_collisions, handle_collisions};
use crate::app::stats::Stats;
use crate::app::Palette;
use crate::apple::spawn::{spawn_apples, SpawnPolicy};
use crate::apple::Apple;
use crate::basic::{CellDim, Dir, HexDim, HexPoint, Point};
use crate::color::Color;
use crate::error::{Error, ErrorConversion, Result};
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::eat_mechanics::{EatBehavior, EatMechanics};
use crate::snake_control::pathfinder;
use crate::view::snakes::OtherSnakes;
use crate::{app, apple, rendering, snake, snake_control};

pub struct DebugScenario {
    env: Environment,
    fps_control: FpsControl,

    offset: Option<Point>,
    fit_to_window: bool,

    seeds: Vec<SnakeBuilder>,

    stats: Stats,
}

// Constructors
#[allow(dead_code)]
impl DebugScenario {
    /// A snake crashes into another snake's body
    pub fn head_body_collision(cell_dim: CellDim) -> Self {
        // snake2 crashes into snake1 coming from the bottom-right

        let seed1 = SnakeBuilder::default()
            .pos(HexPoint { h: 5, v: 7 })
            .dir(Dir::U)
            .len(5)
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::always(EatBehavior::Crash))
            .palette(snake::PaletteTemplate::solid_white_red())
            .controller(snake_control::Template::Programmed(vec![]));

        let seed2 = SnakeBuilder::default()
            .pos(HexPoint { h: 8, v: 7 })
            .dir(Dir::Ul)
            .len(5)
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::always(EatBehavior::Die))
            // .palette(snake::PaletteTemplate::dark_blue_to_red(false))
            .palette(snake::PaletteTemplate::rainbow(true))
            .controller(snake_control::Template::Programmed(vec![]));

        let mut this = Self {
            env: Environment {
                snakes: vec![],
                apples: vec![],
                gtx: GameContext {
                    board_dim: HexDim { h: 20, v: 10 },
                    cell_dim,
                    palette: Palette::dark(),
                    prefs: Default::default(),
                    apple_spawn_policy: SpawnPolicy::None,
                },
                rng: thread_rng(),
            },
            fps_control: FpsControl::new(3.),

            offset: None,
            fit_to_window: false,

            seeds: vec![seed1, seed2],

            stats: Default::default(),
        };
        this.restart();
        this.fps_control.pause();
        this
    }

    /// Head-head dying collision
    pub fn head_head_collision(cell_dim: CellDim) -> Self {
        let seed1 = SnakeBuilder::default()
            .pos(HexPoint { h: 5, v: 7 })
            .dir(Dir::Ur)
            .len(5)
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::always(EatBehavior::Die))
            .palette(snake::PaletteTemplate::solid_white_red())
            .speed(1.0)
            .controller(snake_control::Template::Programmed(vec![]));

        let seed2 = SnakeBuilder::default()
            .pos(HexPoint { h: 11, v: 7 })
            .dir(Dir::Ul)
            .len(5)
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::always(EatBehavior::Die))
            .palette(snake::PaletteTemplate::Solid {
                color: Color::RED,
                eaten: Color::WHITE,
            })
            .speed(1.0)
            .controller(snake_control::Template::Programmed(vec![]));

        let mut this = Self {
            env: Environment {
                snakes: vec![],
                apples: vec![],
                gtx: GameContext {
                    board_dim: HexDim { h: 20, v: 10 },
                    cell_dim,
                    palette: Palette::dark(),
                    prefs: Default::default(),
                    apple_spawn_policy: SpawnPolicy::None,
                },
                rng: thread_rng(),
            },
            fps_control: FpsControl::new(3.),

            offset: None,
            fit_to_window: false,

            seeds: vec![seed1, seed2],

            stats: Default::default(),
        };
        this.restart();
        this.fps_control.pause();
        this
    }

    /// Head-head dying collision where both snakes try to eat the same apple at the same time
    pub fn head_head_collision_apple(cell_dim: CellDim) -> Self {
        let seed1 = SnakeBuilder::default()
            .pos(HexPoint { h: 5, v: 7 })
            .dir(Dir::Ur)
            .len(5)
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::always(EatBehavior::Die))
            .palette(snake::PaletteTemplate::solid_white_red())
            .speed(1.0)
            .controller(snake_control::Template::Programmed(vec![]));

        let seed2 = SnakeBuilder::default()
            .pos(HexPoint { h: 11, v: 7 })
            .dir(Dir::Ul)
            .len(5)
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::always(EatBehavior::Die))
            .palette(snake::PaletteTemplate::Solid {
                color: Color::RED,
                eaten: Color::WHITE,
            })
            .speed(1.0)
            .controller(snake_control::Template::Programmed(vec![]));

        let mut this = Self {
            env: Environment {
                snakes: vec![],
                apples: vec![],
                gtx: GameContext {
                    board_dim: HexDim { h: 20, v: 10 },
                    cell_dim,
                    palette: Palette::dark(),
                    prefs: Default::default(),
                    apple_spawn_policy: SpawnPolicy::None,
                },
                rng: thread_rng(),
            },
            fps_control: FpsControl::new(3.),

            offset: None,
            fit_to_window: false,

            seeds: vec![seed1, seed2],

            stats: Default::default(),
        };
        this.restart();
        this.env.apples = vec![Apple {
            pos: HexPoint { h: 8, v: 6 },
            apple_type: apple::Type::Food(0),
        }];
        this.fps_control.pause();
        this
    }

    /// Stress test
    pub fn many_snakes() -> Self {
        const NUM_SNAKES: usize = 100;

        let rng = &mut thread_rng();
        let seeds: Vec<_> = (0..NUM_SNAKES)
            .map(|i| {
                SnakeBuilder::default()
                    .pos(HexPoint {
                        h: i as isize / 7 * 2 + 3,
                        v: i as isize % 10 * 2 + 3,
                    })
                    .dir(Dir::random(rng))
                    .len(5)
                    .snake_type(snake::Type::Competitor { life: None })
                    .eat_mechanics(EatMechanics::always(EatBehavior::PassOver))
                    .palette(snake::PaletteTemplate::pastel_rainbow(true))
                    // .controller(snake_control::Template::AStar {
                    //     passthrough_knowledge: PassthroughKnowledge::always(false),
                    // })
                    .controller(snake_control::Template::Algorithm(pathfinder::Template::WeightedBFS))
            })
            .collect();

        let mut this = Self {
            env: Environment {
                snakes: vec![],
                apples: vec![],
                gtx: GameContext {
                    board_dim: HexDim { h: 0, v: 0 },
                    cell_dim: Default::default(),
                    palette: app::Palette::dark(),
                    prefs: Prefs::default().special_apples(false),
                    apple_spawn_policy: SpawnPolicy::Random { apple_count: 10 },
                },
                rng: thread_rng(),
            },
            fps_control: FpsControl::new(3.),

            offset: None,
            fit_to_window: true,

            seeds,

            stats: Stats::default(),
        };
        this.restart();
        this.fps_control.pause();
        this
    }

    /// Comparison of persistent and non-persistent skins entering a black hole
    pub fn double_head_body_collision(cell_dim: CellDim) -> Self {
        let wall_seed = SnakeBuilder::default()
            .pos(HexPoint { h: 5, v: 7 })
            .dir(Dir::U)
            .len(15)
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::always(EatBehavior::Crash))
            .palette(snake::PaletteTemplate::solid_white_red())
            .controller(snake_control::Template::Programmed(vec![]));

        let crash_seeds = vec![
            SnakeBuilder::default()
                .pos(HexPoint { h: 14, v: 5 })
                .dir(Dir::Ul)
                .len(5)
                .snake_type(snake::Type::Simulated)
                .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                // .palette(snake::PaletteTemplate::dark_blue_to_red(false))
                .palette(snake::PaletteTemplate::dark_blue_to_red(true))
                .controller(snake_control::Template::Programmed(vec![])),
            SnakeBuilder::default()
                .pos(HexPoint { h: 14, v: 7 })
                .dir(Dir::Ul)
                .len(5)
                .snake_type(snake::Type::Simulated)
                .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                // .palette(snake::PaletteTemplate::dark_blue_to_red(false))
                .palette(snake::PaletteTemplate::dark_blue_to_red(false))
                .controller(snake_control::Template::Programmed(vec![])),
            SnakeBuilder::default()
                .pos(HexPoint { h: 14, v: 9 })
                .dir(Dir::Ul)
                .len(5)
                .snake_type(snake::Type::Simulated)
                .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                // .palette(snake::PaletteTemplate::dark_blue_to_red(false))
                .palette(snake::PaletteTemplate::rainbow(true))
                .controller(snake_control::Template::Programmed(vec![])),
            SnakeBuilder::default()
                .pos(HexPoint { h: 14, v: 11 })
                .dir(Dir::Ul)
                .len(5)
                .snake_type(snake::Type::Simulated)
                .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                // .palette(snake::PaletteTemplate::dark_blue_to_red(false))
                .palette(snake::PaletteTemplate::rainbow(false))
                .controller(snake_control::Template::Programmed(vec![])),
        ];

        let mut this = Self {
            env: Environment {
                snakes: vec![],
                apples: vec![],
                gtx: GameContext {
                    board_dim: HexDim { h: 20, v: 15 },
                    cell_dim,
                    palette: app::Palette::dark(),
                    prefs: Default::default(),
                    apple_spawn_policy: SpawnPolicy::None,
                },
                rng: thread_rng(),
            },
            fps_control: FpsControl::new(3.),

            offset: None,
            fit_to_window: false,

            seeds: iter::once(wall_seed).chain(crash_seeds).collect(),

            stats: Default::default(),
        };
        this.restart();
        this.fps_control.pause();
        this
    }
}

impl DebugScenario {
    fn update_dim(&mut self, ctx: &Context) {
        let gtx = &mut self.env.gtx;
        if self.fit_to_window {
            gtx.board_dim = calculate_board_dim(ctx, gtx.cell_dim);
        }
        self.offset = Some(calculate_offset(ctx, gtx.board_dim, gtx.cell_dim));
    }

    fn restart(&mut self) {
        self.env.snakes = self
            .seeds
            .iter()
            .map(SnakeBuilder::build)
            .map(result::Result::unwrap)
            .collect();
        self.env.apples = vec![];
        self.env.gtx.apple_spawn_policy.reset();
        self.fps_control.pause();
    }

    fn spawn_apples(&mut self) {
        spawn_apples(&mut self.env);
    }

    fn advance_snakes(&mut self, ctx: &Context) {
        advance_snakes(&mut self.env, self.fps_control.context(), ctx);

        let collisions = find_collisions(&self.env);
        let (spawn_snakes, game_over) = handle_collisions(&mut self.env, &collisions);

        if game_over {
            self.fps_control.game_over();
        }

        assert!(spawn_snakes.is_empty(), "unimplemented");

        self.spawn_apples();
    }
}

impl EventHandler<Error> for DebugScenario {
    fn update(&mut self, ctx: &mut Context) -> Result {
        while self.fps_control.can_update() {
            self.advance_snakes(ctx);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result {
        self.fps_control.graphics_frame();

        let mut canvas = Canvas::from_frame(ctx, Some(*Color::BLACK));

        if self.offset.is_none() {
            self.update_dim(ctx)
        }

        let offset = self.offset.unwrap();
        let draw_param = DrawParam::default().dest(offset);

        let env = &mut self.env;

        let grid_mesh = rendering::grid_mesh(&env.gtx, ctx)?;
        canvas.draw(&grid_mesh, draw_param);

        let border_mesh = rendering::border_mesh(&env.gtx, ctx)?;
        canvas.draw(&border_mesh, draw_param);

        let ftx = self.fps_control.context();

        for snake_index in 0..env.snakes.len() {
            let (snake, other_snakes) = OtherSnakes::split_snakes(&mut env.snakes, snake_index);
            snake.update_dir(other_snakes, &env.apples, &env.gtx, ftx, ctx);
        }

        let snake_mesh = rendering::snake_mesh(&mut env.snakes, &env.gtx, ftx, ctx, &mut self.stats)?;
        canvas.draw(&snake_mesh, draw_param);

        if !env.apples.is_empty() {
            let apple_mesh = rendering::apple_mesh(&env.apples, &env.gtx, ftx, ctx, &mut self.stats)?;
            canvas.draw(&apple_mesh, draw_param);
        }

        canvas
            .finish(ctx)
            .map_err(Error::from)
            .with_trace_step("DebugScenario::draw")
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeated: bool) -> Result {
        use fps_control::State::*;

        // TODO: unify controls with other Screens
        if input.keycode == Some(KeyCode::Escape) {
            match self.fps_control.state() {
                Playing => self.fps_control.pause(),
                Paused => self.fps_control.play(),
                GameOver => self.restart(),
            }
        } else if input.keycode == Some(KeyCode::R) {
            self.restart()
        }

        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, _width: f32, _height: f32) -> Result {
        self.update_dim(ctx);
        Ok(())
    }
}
