use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::graphics::{
    DrawParam, {self},
};
use ggez::Context;
use rand::prelude::*;

use crate::app::fps_control::{
    FpsControl, {self},
};
use crate::app::game_context::GameContext;
use crate::app::prefs::Prefs;
use crate::app::screen::board_dim::{calculate_board_dim, calculate_offset};
use crate::app::screen::Environment;
use crate::app::snake_management::{advance_snakes, find_collisions, handle_collisions};
use crate::app::stats::Stats;
use crate::apple::spawn::{spawn_apples, SpawnPolicy};
use crate::apple::Apple;
use crate::basic::{CellDim, Dir, HexDim, HexPoint, Point};
use crate::color::Color;
use crate::error::{AppErrorConversion, AppResult, Error};
use crate::snake::{
    EatBehavior, EatMechanics, PassthroughKnowledge, Snake, {self},
};
use crate::view::snakes::OtherSnakes;
use crate::{app, rendering, snake_control};

pub struct DebugScenario {
    fps_control: FpsControl,

    gtx: GameContext,

    offset: Option<Point>,
    fit_to_window: bool,

    stats: Stats,

    apples: Vec<Apple>,

    seeds: Vec<snake::Builder>,
    snakes: Vec<Snake>,

    rng: ThreadRng,
}

// Constructors
#[allow(dead_code)]
impl DebugScenario {
    /// A snake crashes into another snake's body
    pub fn head_body_collision(cell_dim: CellDim) -> Self {
        // snake2 crashes into snake1 coming from the bottom-right

        let seed1 = snake::Builder::default()
            .pos(HexPoint { h: 5, v: 7 })
            .dir(Dir::U)
            .len(5)
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::always(EatBehavior::Crash))
            .palette(snake::PaletteTemplate::solid_white_red())
            .controller(snake_control::Template::Programmed(vec![]));

        let seed2 = snake::Builder::default()
            .pos(HexPoint { h: 8, v: 7 })
            .dir(Dir::Ul)
            .len(5)
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::always(EatBehavior::Die))
            // .palette(snake::PaletteTemplate::dark_blue_to_red(false))
            .palette(snake::PaletteTemplate::rainbow(true))
            .controller(snake_control::Template::Programmed(vec![]));

        let mut this = Self {
            fps_control: FpsControl::new(3.),

            gtx: GameContext {
                board_dim: HexDim { h: 20, v: 10 },
                cell_dim,
                palette: app::Palette::dark(),
                prefs: Default::default(),
                apple_spawn_policy: SpawnPolicy::None,
                frame_stamp: (0, 0.0),
                game_frame_num: 0,
                elapsed_millis: 0,
            },

            offset: None,
            fit_to_window: false,

            stats: Default::default(),

            apples: vec![],

            seeds: vec![seed1, seed2],
            snakes: vec![],

            rng: thread_rng(),
        };
        this.restart();
        this.fps_control.pause();
        this
    }

    /// Head-head dying collision
    pub fn head_head_collision(cell_dim: CellDim) -> Self {
        let seed1 = snake::Builder::default()
            .pos(HexPoint { h: 5, v: 7 })
            .dir(Dir::Ur)
            .len(5)
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::always(EatBehavior::Die))
            .palette(snake::PaletteTemplate::solid_white_red())
            .controller(snake_control::Template::Programmed(vec![]));

        let seed2 = snake::Builder::default()
            .pos(HexPoint { h: 11, v: 7 })
            .dir(Dir::Ul)
            .len(5)
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::always(EatBehavior::Die))
            .palette(snake::PaletteTemplate::Solid {
                color: Color::RED,
                eaten: Color::WHITE,
            })
            .controller(snake_control::Template::Programmed(vec![]));

        let mut this = Self {
            fps_control: FpsControl::new(3.),

            gtx: GameContext {
                board_dim: HexDim { h: 20, v: 10 },
                cell_dim,
                palette: app::Palette::dark(),
                prefs: Default::default(),
                apple_spawn_policy: SpawnPolicy::None,
                frame_stamp: (0, 0.0),
                game_frame_num: 0,
                elapsed_millis: 0,
            },

            offset: None,
            fit_to_window: false,

            stats: Default::default(),

            apples: vec![],

            seeds: vec![seed1, seed2],
            snakes: vec![],

            rng: thread_rng(),
        };
        this.restart();
        this.fps_control.pause();
        this
    }

    /// Stress test
    pub fn many_snakes() -> Self {
        const NUM_SNAKES: usize = 100;

        let rng = &mut thread_rng();
        let seeds: Vec<_> = (0..NUM_SNAKES)
            .map(|i| {
                snake::Builder::default()
                    .pos(HexPoint {
                        h: i as isize / 7 * 2 + 3,
                        v: i as isize % 10 * 2 + 3,
                    })
                    .dir(Dir::random(rng))
                    .len(5)
                    .snake_type(snake::Type::Competitor { life: None })
                    .eat_mechanics(EatMechanics::always(EatBehavior::PassOver))
                    .palette(snake::PaletteTemplate::pastel_rainbow(true))
                    .controller(snake_control::Template::AStar {
                        passthrough_knowledge: PassthroughKnowledge::always(false),
                    })
            })
            .collect();

        let mut this = Self {
            fps_control: FpsControl::new(3.),

            gtx: GameContext {
                board_dim: HexDim { h: 0, v: 0 },
                cell_dim: Default::default(),
                palette: app::Palette::dark(),
                prefs: Prefs::default().special_apples(false),
                apple_spawn_policy: SpawnPolicy::Random { apple_count: 10 },
                frame_stamp: (0, 0.0),
                game_frame_num: 0,
                elapsed_millis: 0,
            },

            offset: None,
            fit_to_window: true,

            stats: Stats::default(),

            apples: vec![],

            seeds,
            snakes: vec![],

            rng: thread_rng(),
        };
        this.restart();
        this.fps_control.pause();
        this
    }

    /// Comparison of persistent and non-persistent skins entering a black hole
    pub fn double_head_body_collision(cell_dim: CellDim) -> Self {
        let wall_seed = snake::Builder::default()
            .pos(HexPoint { h: 5, v: 7 })
            .dir(Dir::U)
            .len(15)
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::always(EatBehavior::Crash))
            .palette(snake::PaletteTemplate::solid_white_red())
            .controller(snake_control::Template::Programmed(vec![]));

        let crash_seeds = vec![
            snake::Builder::default()
                .pos(HexPoint { h: 14, v: 5 })
                .dir(Dir::Ul)
                .len(5)
                .snake_type(snake::Type::Simulated)
                .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                // .palette(snake::PaletteTemplate::dark_blue_to_red(false))
                .palette(snake::PaletteTemplate::dark_blue_to_red(true))
                .controller(snake_control::Template::Programmed(vec![])),
            snake::Builder::default()
                .pos(HexPoint { h: 14, v: 7 })
                .dir(Dir::Ul)
                .len(5)
                .snake_type(snake::Type::Simulated)
                .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                // .palette(snake::PaletteTemplate::dark_blue_to_red(false))
                .palette(snake::PaletteTemplate::dark_blue_to_red(false))
                .controller(snake_control::Template::Programmed(vec![])),
            snake::Builder::default()
                .pos(HexPoint { h: 14, v: 9 })
                .dir(Dir::Ul)
                .len(5)
                .snake_type(snake::Type::Simulated)
                .eat_mechanics(EatMechanics::always(EatBehavior::Die))
                // .palette(snake::PaletteTemplate::dark_blue_to_red(false))
                .palette(snake::PaletteTemplate::rainbow(true))
                .controller(snake_control::Template::Programmed(vec![])),
            snake::Builder::default()
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
            fps_control: FpsControl::new(3.),

            gtx: GameContext {
                board_dim: HexDim { h: 20, v: 15 },
                cell_dim,
                palette: app::Palette::dark(),
                prefs: Default::default(),
                apple_spawn_policy: SpawnPolicy::None,
                frame_stamp: (0, 0.0),
                game_frame_num: 0,
                elapsed_millis: 0,
            },

            offset: None,
            fit_to_window: false,

            stats: Default::default(),

            apples: vec![],

            seeds: vec![wall_seed].into_iter().chain(crash_seeds).collect(),
            snakes: vec![],

            rng: thread_rng(),
        };
        this.restart();
        this.fps_control.pause();
        this
    }
}

impl DebugScenario {
    fn update_dim(&mut self, ctx: &Context) {
        if self.fit_to_window {
            self.gtx.board_dim = calculate_board_dim(ctx, self.gtx.cell_dim);
        }
        self.offset = Some(calculate_offset(ctx, self.gtx.board_dim, self.gtx.cell_dim));
    }

    fn restart(&mut self) {
        self.snakes = self
            .seeds
            .iter()
            .map(snake::Builder::build)
            .map(Result::unwrap)
            .collect();
        self.apples = vec![];
        self.gtx.apple_spawn_policy.reset();
        self.fps_control.pause();
    }

    fn spawn_apples(&mut self) {
        let new_apples = spawn_apples(&self.snakes, &self.apples, &mut self.gtx, &mut self.rng);
        self.apples.extend(new_apples.into_iter())
    }

    fn advance_snakes(&mut self, ctx: &Context) {
        advance_snakes(self, ctx);

        let collisions = find_collisions(self);
        let (spawn_snakes, game_over) = handle_collisions(self, &collisions);

        if game_over {
            self.fps_control.game_over();
        }

        assert!(spawn_snakes.is_empty(), "unimplemented");

        self.spawn_apples();
    }
}

impl EventHandler<Error> for DebugScenario {
    fn update(&mut self, ctx: &mut Context) -> AppResult {
        while self.fps_control.can_update(&mut self.gtx) {
            self.advance_snakes(ctx);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> AppResult {
        self.fps_control.graphics_frame(&mut self.gtx);

        graphics::clear(ctx, *Color::BLACK);

        if self.offset.is_none() {
            self.update_dim(ctx)
        }

        let offset = self.offset.unwrap();
        let draw_param = DrawParam::default().dest(offset);

        let grid_mesh = rendering::grid_mesh(&self.gtx, ctx)?;
        graphics::draw(ctx, &grid_mesh, draw_param)?;

        let border_mesh = rendering::border_mesh(&self.gtx, ctx)?;
        graphics::draw(ctx, &border_mesh, draw_param)?;

        for snake_index in 0..self.snakes.len() {
            let (snake, other_snakes) = OtherSnakes::split_snakes(&mut self.snakes, snake_index);
            snake.update_dir(other_snakes, &self.apples, &self.gtx, ctx);
        }

        let snake_mesh = rendering::snake_mesh(&mut self.snakes, &self.gtx, ctx, &mut self.stats)?;
        graphics::draw(ctx, &snake_mesh, draw_param)?;

        if !self.apples.is_empty() {
            let apple_mesh = rendering::apple_mesh(&self.apples, &self.gtx, ctx, &mut self.stats)?;
            graphics::draw(ctx, &apple_mesh, draw_param)?;
        }

        graphics::present(ctx)
            .map_err(Error::from)
            .with_trace_step("DebugScenario::draw")
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        use fps_control::State::*;

        if keycode == KeyCode::Space {
            match self.fps_control.state() {
                Playing => self.fps_control.pause(),
                Paused => self.fps_control.play(),
                GameOver => self.restart(),
            }
        } else if keycode == KeyCode::R {
            self.restart()
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, _width: f32, _height: f32) {
        self.update_dim(ctx)
    }
}

impl Environment for DebugScenario {
    fn snakes(&self) -> &[Snake] {
        &self.snakes
    }

    fn apples(&self) -> &[Apple] {
        &self.apples
    }

    fn snakes_apples_gtx_mut(&mut self) -> (&mut [Snake], &mut [Apple], &mut GameContext) {
        (&mut self.snakes, &mut self.apples, &mut self.gtx)
    }

    fn snakes_apples_rng_mut(&mut self) -> (&mut [Snake], &mut [Apple], &mut ThreadRng) {
        (&mut self.snakes, &mut self.apples, &mut self.rng)
    }

    fn add_snake(&mut self, snake_builder: &snake::Builder) -> AppResult {
        self.snakes.push(
            snake_builder
                .build()
                .map_err(Error::from)
                .with_trace_step("Game::add_snake")?,
        );
        Ok(())
    }

    fn remove_snake(&mut self, index: usize) -> Snake {
        self.snakes.remove(index)
    }

    fn remove_apple(&mut self, index: usize) -> Apple {
        self.apples.remove(index)
    }

    fn gtx(&self) -> &GameContext {
        &self.gtx
    }

    fn rng(&mut self) -> &mut ThreadRng {
        &mut self.rng
    }
}
