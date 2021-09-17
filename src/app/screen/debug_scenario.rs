use ggez::{Context, GameResult, GameError};
use ggez::event::{EventHandler, MouseButton, KeyMods, Button, GamepadId, Axis, ErrorOrigin, KeyCode};
use ggez::graphics::{self, Color, DrawParam};
use rand::prelude::*;

use crate::app;
use crate::app::apple::spawn::{spawn_apples, SpawnPolicy};
use crate::app::snake_management::{find_collisions, handle_collisions, advance_snakes};
use crate::app::control::{self, Control};
use crate::app::prefs::Prefs;
use crate::app::rendering;
use crate::app::apple::Apple;
use crate::app::snake::{self, EatBehavior, EatMechanics, Snake};
use crate::app::snake::controller::Template;
use crate::app::snake::utils::split_snakes_mut;
use crate::app::stats::Stats;
use crate::basic::{CellDim, Dir, HexDim, HexPoint, Point};
use crate::basic::FrameStamp;

pub struct DebugScenario {
    control: Control,

    cell_dim: CellDim,

    board_dim: HexDim,
    offset: Point,
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
    pub fn collision1(cell_dim: CellDim) -> Self {
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
            palette: snake::PaletteTemplate::Solid { color: Color::RED, eaten: Color::WHITE },
            controller: Template::Programmed(vec![]),
        };

        let snake1 = Snake::from_seed(&seed1);
        let snake2 = Snake::from_seed(&seed2);

        let mut this = Self {
            control: Control::new(3.),

            cell_dim,

            board_dim: HexDim { h: 20, v: 10 },
            offset: Point { x: 0., y: 0. },
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
        let new_apples = spawn_apples(&mut self.apple_spawn_policy, self.board_dim, &self.snakes, &self.apples, &self.prefs, &mut self.rng);
        self.apples.extend(new_apples.into_iter())
    }

    fn advance_snakes(&mut self, frame_stamp: FrameStamp) {
        advance_snakes(&mut self.snakes, &self.apples, self.board_dim, self.control.frame_stamp());

        let collisions = find_collisions(&self.snakes, &self.apples);
        let (spawn_snakes, remove_apples, game_over) =
            handle_collisions(&collisions, &mut self.snakes, &self.apples);

        if game_over { self.control.game_over(); }

        for apple_index in remove_apples.into_iter().rev() {
            self.apples.remove(apple_index);
        }

        assert!(spawn_snakes.is_empty(), "unimplemented");

        self.spawn_apples();
    }
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

        let draw_param = DrawParam::default().dest(self.offset);

        let grid_mesh = rendering::grid_mesh(self.board_dim, self.cell_dim, &self.palette, ctx)?;
        graphics::draw(ctx, &grid_mesh, draw_param)?;

        let border_mesh = rendering::border_mesh(self.board_dim, self.cell_dim, &self.palette, ctx)?;
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

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        if keycode == KeyCode::Space {
            match self.control.state() {
                control::State::Playing => { self.control.pause() }
                control::State::Paused => { self.control.play() }
                control::State::GameOver => {}
            }
        }
    }
}
