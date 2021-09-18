use ggez::{
    event::{EventHandler, KeyCode, KeyMods},
    graphics::{self, Color, DrawParam},
    Context, GameError, GameResult,
};
use rand::prelude::*;

use crate::{
    app,
    app::{
        apple::{
            spawn::{spawn_apples, SpawnPolicy},
            Apple,
        },
        control::{self, Control},
        prefs::Prefs,
        rendering,
        screen::Environment,
        snake::{
            self, controller::Template, utils::split_snakes_mut, EatBehavior, EatMechanics, Seed,
            Snake,
        },
        snake_management::{advance_snakes, find_collisions, handle_collisions},
        stats::Stats,
    },
    basic::{CellDim, Dir, FrameStamp, HexDim, HexPoint, Point},
};
use ggez::event::{Axis, Button, ErrorOrigin, GamepadId, MouseButton};

pub struct DebugScenario {
    control: Control,

    cell_dim: CellDim,

    board_dim: HexDim,
    offset: Option<Point>,
    fit_to_window: bool,

    palette: app::Palette,
    prefs: Prefs,
    stats: Stats,

    apples: Vec<Apple>,
    apple_spawn_policy: SpawnPolicy,

    seeds: Vec<snake::Seed>,
    snakes: Vec<Snake>,

    rng: ThreadRng,
}

impl DebugScenario {
    /// A snake crashes into another snake's body
    pub fn head_body_collision(cell_dim: CellDim) -> Self {
        // snake2 crashes into snake1 coming from the bottom-right

        let seed1 = snake::Seed {
            pos: Some(HexPoint { h: 5, v: 7 }),
            dir: Some(Dir::U),
            len: Some(5),

            snake_type: snake::Type::Simulated,
            eat_mechanics: EatMechanics {
                eat_self: EatBehavior::Crash,
                eat_other: hash_map! {},
                default: EatBehavior::Crash,
            },
            palette: snake::PaletteTemplate::solid_white_red(),
            controller: Template::Programmed(vec![]),
        };

        let seed2 = snake::Seed {
            pos: Some(HexPoint { h: 8, v: 7 }),
            dir: Some(Dir::Ul),
            len: Some(5),

            snake_type: snake::Type::Simulated,
            eat_mechanics: EatMechanics {
                eat_self: EatBehavior::Crash,
                eat_other: hash_map! {},
                default: EatBehavior::Die,
            },
            // palette: snake::PaletteTemplate::dark_blue_to_red(false),
            palette: snake::PaletteTemplate::rainbow(true),
            controller: Template::Programmed(vec![]),
        };

        let snake1 = Snake::from(&seed1);
        let snake2 = Snake::from(&seed2);

        let mut this = Self {
            control: Control::new(3.),

            cell_dim,

            board_dim: HexDim { h: 20, v: 10 },
            offset: None,
            fit_to_window: false,

            palette: app::Palette::dark(),
            prefs: Default::default(),
            stats: Default::default(),

            apples: vec![],
            apple_spawn_policy: SpawnPolicy::None,

            seeds: vec![seed1, seed2],
            snakes: vec![snake1, snake2],

            rng: thread_rng(),
        };
        this.control.pause();
        this
    }

    /// Head-head collision
    pub fn head_head_collision(cell_dim: CellDim) -> Self {
        let seed1 = snake::Seed {
            pos: Some(HexPoint { h: 5, v: 7 }),
            dir: Some(Dir::Ur),
            len: Some(5),

            snake_type: snake::Type::Simulated,
            eat_mechanics: EatMechanics {
                eat_self: EatBehavior::Crash,
                eat_other: hash_map! {},
                default: EatBehavior::Die,
            },
            palette: snake::PaletteTemplate::solid_white_red(),
            controller: Template::Programmed(vec![]),
        };

        let seed2 = snake::Seed {
            pos: Some(HexPoint { h: 11, v: 7 }),
            dir: Some(Dir::Ul),
            len: Some(5),

            snake_type: snake::Type::Simulated,
            eat_mechanics: EatMechanics {
                eat_self: EatBehavior::Crash,
                eat_other: hash_map! {},
                default: EatBehavior::Die,
            },
            palette: snake::PaletteTemplate::Solid {
                color: Color::RED,
                eaten: Color::WHITE,
            },
            controller: Template::Programmed(vec![]),
        };

        let snake1 = Snake::from(&seed1);
        let snake2 = Snake::from(&seed2);

        let mut this = Self {
            control: Control::new(3.),

            cell_dim,

            board_dim: HexDim { h: 20, v: 10 },
            offset: None,
            fit_to_window: false,

            palette: app::Palette::dark(),
            prefs: Default::default(),
            stats: Default::default(),

            apples: vec![],
            apple_spawn_policy: SpawnPolicy::None,

            seeds: vec![seed1, seed2],
            snakes: vec![snake1, snake2],

            rng: thread_rng(),
        };
        this.control.pause();
        this
    }

    fn spawn_apples(&mut self) {
        let new_apples = spawn_apples(
            &mut self.apple_spawn_policy,
            self.board_dim,
            &self.snakes,
            &self.apples,
            &self.prefs,
            &mut self.rng,
        );
        self.apples.extend(new_apples.into_iter())
    }

    fn advance_snakes(&mut self, _frame_stamp: FrameStamp) {
        advance_snakes(self);

        let collisions = find_collisions(self);
        let (spawn_snakes, game_over) = handle_collisions(self, &collisions);

        if game_over {
            self.control.game_over();
        }

        assert!(spawn_snakes.is_empty(), "unimplemented");

        self.spawn_apples();
    }
}

fn get_offset(board_dim: HexDim, window_dim: Point, cell_dim: CellDim) -> Point {
    let CellDim { side, sin, cos } = cell_dim;

    let board_cartesian_dim = Point {
        x: board_dim.h as f32 * (side + cos) + cos,
        y: board_dim.v as f32 * 2. * sin + sin,
    };
    (window_dim - board_cartesian_dim) / 2.
}

impl EventHandler<ggez::GameError> for DebugScenario {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        while self.control.can_update() {
            let frame_stamp = self.control.frame_stamp();
            self.advance_snakes(frame_stamp);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.control.graphics_frame();
        let frame_stamp = self.control.frame_stamp();

        graphics::clear(ctx, Color::BLACK);

        let offset = *self.offset.get_or_insert_with(|| {
            let window_dim = ggez::graphics::window(ctx).inner_size();
            let window_dim = Point {
                x: window_dim.width as f32,
                y: window_dim.height as f32,
            };
            get_offset(self.board_dim, window_dim, self.cell_dim)
        });
        let draw_param = DrawParam::default().dest(offset);

        let grid_mesh = rendering::grid_mesh(self.board_dim, self.cell_dim, &self.palette, ctx)?;
        graphics::draw(ctx, &grid_mesh, draw_param)?;

        let border_mesh =
            rendering::border_mesh(self.board_dim, self.cell_dim, &self.palette, ctx)?;
        graphics::draw(ctx, &border_mesh, draw_param)?;

        for snake_index in 0..self.snakes.len() {
            let (snake, other_snakes) = split_snakes_mut(&mut self.snakes, snake_index);
            snake.update_dir(other_snakes, &self.apples, self.board_dim, frame_stamp);
        }

        let snake_mesh = rendering::snake_mesh(
            &mut self.snakes,
            frame_stamp,
            self.board_dim,
            self.cell_dim,
            self.prefs.draw_style,
            ctx,
            &mut self.stats,
        )?;
        graphics::draw(ctx, &snake_mesh, draw_param)?;

        if !self.apples.is_empty() {
            let apple_mesh = rendering::apple_mesh(
                &self.apples,
                frame_stamp,
                self.cell_dim,
                self.prefs.draw_style,
                &self.palette,
                ctx,
                &mut self.stats,
            )?;
            graphics::draw(ctx, &apple_mesh, draw_param)?;
        }

        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if keycode == KeyCode::Space {
            match self.control.state() {
                control::State::Playing => self.control.pause(),
                control::State::Paused => self.control.play(),
                control::State::GameOver => {}
            }
        }
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) {
        self.offset = Some(get_offset(
            self.board_dim,
            Point { x: width, y: height },
            self.cell_dim,
        ));
    }
}

impl Environment for DebugScenario {
    fn snakes(&self) -> &[Snake] {
        &self.snakes
    }

    fn apples(&self) -> &[Apple] {
        &self.apples
    }

    fn snakes_apples_mut(&mut self) -> (&mut [Snake], &mut [Apple]) {
        (&mut self.snakes, &mut self.apples)
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

    fn board_dim(&self) -> HexDim {
        self.board_dim
    }

    fn frame_stamp(&self) -> FrameStamp {
        self.control.frame_stamp()
    }

    fn rng(&mut self) -> &mut ThreadRng {
        &mut self.rng
    }
}
