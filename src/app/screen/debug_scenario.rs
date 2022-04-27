use ggez::{
    event::{EventHandler, KeyCode, KeyMods},
    graphics::{self, Color, DrawParam},
    Context,
};
use rand::prelude::*;

use crate::{
    app,
    app::{
        app_error::{AppError, AppResult, GameResultExtension},
        apple::{
            spawn::{spawn_apples, SpawnPolicy},
            Apple,
        },
        control::{self, Control},
        game_context::GameContext,
        prefs::Prefs,
        rendering,
        screen::{
            board_dim::{calculate_board_dim, calculate_offset},
            Environment,
        },
        snake::{
            self, controller, utils::split_snakes_mut, EatBehavior, EatMechanics, Seed, Snake,
        },
        snake_management::{advance_snakes, find_collisions, handle_collisions},
        stats::Stats,
    },
    basic::{CellDim, Dir, HexDim, HexPoint, Point},
};

pub struct DebugScenario {
    control: Control,

    gtx: GameContext,

    offset: Option<Point>,
    fit_to_window: bool,

    stats: Stats,

    apples: Vec<Apple>,

    seeds: Vec<snake::Seed>,
    snakes: Vec<Snake>,

    rng: ThreadRng,
}

// Constructors
#[allow(dead_code)]
impl DebugScenario {
    /// A snake crashes into another snake's body
    pub fn head_body_collision(cell_dim: CellDim) -> Self {
        // snake2 crashes into snake1 coming from the bottom-right

        let seed1 = snake::Seed {
            pos: Some(HexPoint { h: 5, v: 7 }),
            dir: Some(Dir::U),
            len: Some(5),

            snake_type: snake::Type::Simulated,
            eat_mechanics: EatMechanics::always(EatBehavior::Crash),
            palette: snake::PaletteTemplate::solid_white_red(),
            controller: controller::Template::Programmed(vec![]),
        };

        let seed2 = snake::Seed {
            pos: Some(HexPoint { h: 8, v: 7 }),
            dir: Some(Dir::Ul),
            len: Some(5),

            snake_type: snake::Type::Simulated,
            eat_mechanics: EatMechanics::always(EatBehavior::Die),
            // palette: snake::PaletteTemplate::dark_blue_to_red(false),
            palette: snake::PaletteTemplate::rainbow(true),
            controller: controller::Template::Programmed(vec![]),
        };

        let mut this = Self {
            control: Control::new(3.),

            gtx: GameContext {
                board_dim: HexDim { h: 20, v: 10 },
                cell_dim,
                palette: app::Palette::dark(),
                prefs: Default::default(),
                apple_spawn_policy: SpawnPolicy::None,
                frame_stamp: Default::default(),
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
        this.control.pause();
        this
    }

    /// Head-head dying collision
    pub fn head_head_collision(cell_dim: CellDim) -> Self {
        let seed1 = snake::Seed {
            pos: Some(HexPoint { h: 5, v: 7 }),
            dir: Some(Dir::Ur),
            len: Some(5),

            snake_type: snake::Type::Simulated,
            eat_mechanics: EatMechanics::always(EatBehavior::Die),
            palette: snake::PaletteTemplate::solid_white_red(),
            controller: controller::Template::Programmed(vec![]),
        };

        let seed2 = snake::Seed {
            pos: Some(HexPoint { h: 11, v: 7 }),
            dir: Some(Dir::Ul),
            len: Some(5),

            snake_type: snake::Type::Simulated,
            eat_mechanics: EatMechanics::always(EatBehavior::Die),
            palette: snake::PaletteTemplate::Solid {
                color: Color::RED,
                eaten: Color::WHITE,
            },
            controller: controller::Template::Programmed(vec![]),
        };

        let mut this = Self {
            control: Control::new(3.),

            gtx: GameContext {
                board_dim: HexDim { h: 20, v: 10 },
                cell_dim,
                palette: app::Palette::dark(),
                prefs: Default::default(),
                apple_spawn_policy: SpawnPolicy::None,
                frame_stamp: Default::default(),
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
        this.control.pause();
        this
    }

    /// Stress test
    pub fn many_snakes() -> Self {
        const NUM_SNAKES: usize = 100;

        let rng = &mut thread_rng();
        let seeds: Vec<_> = (0..NUM_SNAKES)
            .map(|i| snake::Seed {
                pos: Some(HexPoint {
                    h: i as isize / 7 * 2 + 3,
                    v: i as isize % 10 * 2 + 3,
                }),
                dir: Some(Dir::random(rng)),
                len: Some(5),

                snake_type: snake::Type::Competitor { life: None },
                eat_mechanics: EatMechanics::always(EatBehavior::PassOver),
                palette: snake::PaletteTemplate::pastel_rainbow(true),
                controller: controller::Template::AStar,
            })
            .collect();

        let mut this = Self {
            control: Control::new(3.),

            gtx: GameContext {
                board_dim: HexDim { h: 0, v: 0 },
                cell_dim: Default::default(),
                palette: app::Palette::dark(),
                prefs: Prefs::default().special_apples(false),
                apple_spawn_policy: SpawnPolicy::Random { apple_count: 10 },
                frame_stamp: Default::default(),
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
        this.control.pause();
        this
    }

    /// Comparison of persistent and non-persistent skins entering a black hole
    pub fn double_head_body_collision(cell_dim: CellDim) -> Self {
        let wall_seed = snake::Seed {
            pos: Some(HexPoint { h: 5, v: 7 }),
            dir: Some(Dir::U),
            len: Some(15),

            snake_type: snake::Type::Simulated,
            eat_mechanics: EatMechanics::always(EatBehavior::Crash),
            palette: snake::PaletteTemplate::solid_white_red(),
            controller: controller::Template::Programmed(vec![]),
        };

        let crash_seeds = vec![
            snake::Seed {
                pos: Some(HexPoint { h: 14, v: 5 }),
                dir: Some(Dir::Ul),
                len: Some(5),

                snake_type: snake::Type::Simulated,
                eat_mechanics: EatMechanics::always(EatBehavior::Die),
                // palette: snake::PaletteTemplate::dark_blue_to_red(false),
                palette: snake::PaletteTemplate::dark_blue_to_red(true),
                controller: controller::Template::Programmed(vec![]),
            },
            snake::Seed {
                pos: Some(HexPoint { h: 14, v: 7 }),
                dir: Some(Dir::Ul),
                len: Some(5),

                snake_type: snake::Type::Simulated,
                eat_mechanics: EatMechanics::always(EatBehavior::Die),
                // palette: snake::PaletteTemplate::dark_blue_to_red(false),
                palette: snake::PaletteTemplate::dark_blue_to_red(false),
                controller: controller::Template::Programmed(vec![]),
            },
            snake::Seed {
                pos: Some(HexPoint { h: 14, v: 9 }),
                dir: Some(Dir::Ul),
                len: Some(5),

                snake_type: snake::Type::Simulated,
                eat_mechanics: EatMechanics::always(EatBehavior::Die),
                // palette: snake::PaletteTemplate::dark_blue_to_red(false),
                palette: snake::PaletteTemplate::rainbow(true),
                controller: controller::Template::Programmed(vec![]),
            },
            snake::Seed {
                pos: Some(HexPoint { h: 14, v: 11 }),
                dir: Some(Dir::Ul),
                len: Some(5),

                snake_type: snake::Type::Simulated,
                eat_mechanics: EatMechanics::always(EatBehavior::Die),
                // palette: snake::PaletteTemplate::dark_blue_to_red(false),
                palette: snake::PaletteTemplate::rainbow(false),
                controller: controller::Template::Programmed(vec![]),
            },
        ];

        let mut this = Self {
            control: Control::new(3.),

            gtx: GameContext {
                board_dim: HexDim { h: 20, v: 15 },
                cell_dim,
                palette: app::Palette::dark(),
                prefs: Default::default(),
                apple_spawn_policy: SpawnPolicy::None,
                frame_stamp: Default::default(),
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
        this.control.pause();
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
        self.snakes = self.seeds.iter().map(Snake::from).collect();
        self.apples = vec![];
        self.gtx.apple_spawn_policy.reset();
        self.control.pause();
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
            self.control.game_over();
        }

        assert!(spawn_snakes.is_empty(), "unimplemented");

        self.spawn_apples();
    }
}

impl EventHandler<AppError> for DebugScenario {
    fn update(&mut self, ctx: &mut Context) -> AppResult {
        while self.control.can_update() {
            self.advance_snakes(ctx);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> AppResult {
        self.control.graphics_frame(&mut self.gtx);

        graphics::clear(ctx, Color::BLACK);

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
            let (snake, other_snakes) = split_snakes_mut(&mut self.snakes, snake_index);
            snake.update_dir(other_snakes, &self.apples, &self.gtx, ctx);
        }

        let snake_mesh = rendering::snake_mesh(&mut self.snakes, &self.gtx, ctx, &mut self.stats)?;
        graphics::draw(ctx, &snake_mesh, draw_param)?;

        if !self.apples.is_empty() {
            let apple_mesh = rendering::apple_mesh(&self.apples, &self.gtx, ctx, &mut self.stats)?;
            graphics::draw(ctx, &apple_mesh, draw_param)?;
        }

        graphics::present(ctx).into_with_trace("DebugScenario::draw")
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        use control::State::*;

        if keycode == KeyCode::Space {
            match self.control.state() {
                Playing => self.control.pause(),
                Paused => self.control.play(),
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

    fn add_snake(&mut self, seed: &Seed) {
        self.snakes.push(Snake::from(seed))
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
