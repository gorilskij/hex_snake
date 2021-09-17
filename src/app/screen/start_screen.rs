use std::slice;

use ggez::{
    Context,
    event::EventHandler,
    GameResult, graphics::{self, DrawParam},
};
use ggez::{
    event::{KeyCode, KeyMods},
    graphics::Color,
};
use rand::prelude::*;
use rand::rngs::ThreadRng;

use crate::{
    app,
    app::{
        apple::{self, Apple
        },
        snake_management::{find_collisions, handle_collisions},
        Screen,
        snake,
        snake::{
            controller::Template,
            EatBehavior, EatMechanics, Seed, Snake, utils::OtherSnakes,
        },
    },
    basic::{CellDim, Dir, HexDim, HexPoint},
};
use crate::app::apple::spawn::{ScheduledSpawn, spawn_apples, SpawnPolicy};
use crate::app::control::Control;
use crate::app::prefs::Prefs;
use crate::app::rendering;
use crate::app::stats::Stats;
use crate::basic::FrameStamp;

// position of the snake within the demo box is relative,
// the snake thinks it's in an absolute world at (0, 0)
struct SnakeDemo {
    location: HexPoint, // top-left
    board_dim: HexDim,
    apples: Vec<Apple>,
    apple_spawn_policy: SpawnPolicy,
    snake: Snake,
    palettes: Vec<snake::PaletteTemplate>,
    current_palette: usize,
}

impl SnakeDemo {
    fn new(location: HexPoint) -> Self {
        let board_dim = HexPoint { h: 11, v: 8 };
        let start_pos = HexPoint { h: 1, v: 4 };
        let start_dir = Dir::U;
        let start_len = 5;

        let spawn_schedule = spawn_schedule![spawn(6, 2), wait(40),];
        let apple_spawn_policy = SpawnPolicy::ScheduledOnEat {
            apple_count: 1,
            spawns: spawn_schedule,
            next_index: 0,
        };

        let mut seed = Seed {
            snake_type: snake::Type::Simulated,
            eat_mechanics: EatMechanics {
                eat_self: EatBehavior::Cut,
                eat_other: hash_map! {},
                default: EatBehavior::Cut,
            },
            // placeholder, updated immediately
            palette: snake::PaletteTemplate::Solid {
                color: Color::RED,
                eaten: Color::RED,
            },
            controller: Template::demo_infinity_pattern(1),
            pos: Some(start_pos),
            dir: Some(start_dir),
            len: Some(start_len),
        };

        let palettes = vec![
            snake::PaletteTemplate::solid_white_red(),
            // snake::PaletteTemplate::rainbow(false),
            snake::PaletteTemplate::rainbow(true),
            snake::PaletteTemplate::alternating_white(),
            snake::PaletteTemplate::gray_gradient(false),
            snake::PaletteTemplate::green_to_red(false),
            // snake::PaletteTemplate::zebra(),
        ];
        seed.palette = palettes[0];

        Self {
            location,
            board_dim,
            apples: vec![],
            apple_spawn_policy,
            snake: Snake::from_seed(&seed),
            palettes,
            current_palette: 0,
        }
    }
}

impl SnakeDemo {
    fn next_palette(&mut self) {
        self.current_palette = (self.current_palette + 1) % self.palettes.len();
        self.snake.palette = self.palettes[self.current_palette].into();
    }

    fn spawn_apples(&mut self, prefs: &Prefs, rng: &mut impl Rng) {
        let new_apples = spawn_apples(&mut self.apple_spawn_policy, self.board_dim, slice::from_ref(&self.snake), &self.apples, prefs, rng);
        self.apples.extend(new_apples.into_iter());
    }

    fn advance_snakes(&mut self, frame_stamp: FrameStamp, prefs: &Prefs, rng: &mut impl Rng) {
        self.snake
            .advance(OtherSnakes::empty(), &self.apples, self.board_dim, frame_stamp);

        let collisions = find_collisions(slice::from_ref(&self.snake), &self.apples);
        let (spawn_snakes, remove_apples, game_over) =
            handle_collisions(&collisions, slice::from_mut(&mut self.snake), &self.apples);

        assert!(spawn_snakes.is_empty(), "unexpected snake spawn");
        assert_eq!(game_over, false, "unexpected game over");
        for apple_index in remove_apples.into_iter().rev() {
            self.apples.remove(apple_index);
        }

        self.spawn_apples(prefs, rng);
    }

    fn draw(
        &mut self,
        ctx: &mut Context,
        cell_dim: CellDim,
        frame_stamp: FrameStamp,
        draw_style: rendering::Style,
        palette: &app::Palette,
        stats: &mut Stats,
    ) -> GameResult {
        self.snake
            .update_dir(OtherSnakes::empty(), &[], self.board_dim, frame_stamp);

        let draw_param = DrawParam::default().dest(self.location.to_cartesian(cell_dim));

        let grid_mesh = rendering::grid_mesh(self.board_dim, cell_dim, palette, ctx)?;
        graphics::draw(ctx, &grid_mesh, draw_param)?;

        let border_mesh = rendering::border_mesh(self.board_dim, cell_dim, palette, ctx)?;
        graphics::draw(ctx, &border_mesh, draw_param)?;

        let snake_mesh = rendering::snake_mesh(
            slice::from_mut(&mut self.snake),
            frame_stamp,
            self.board_dim,
            cell_dim,
            draw_style,
            ctx,
            stats,
        )?;
        graphics::draw(ctx, &snake_mesh, draw_param)?;

        if !self.apples.is_empty() {
            let apple_mesh = rendering::apple_mesh(
                &self.apples,
                frame_stamp,
                cell_dim,
                draw_style,
                palette,
                ctx,
                stats,
            )?;
            graphics::draw(ctx, &apple_mesh, draw_param)?;
        }

        Ok(())
    }
}

pub struct StartScreen {
    control: Control,
    cell_dim: CellDim,

    palettes: Vec<app::Palette>,
    current_palette: usize,

    prefs: Prefs,
    stats: Stats,

    player1_demo: SnakeDemo,
    player2_demo: SnakeDemo,

    rng: ThreadRng,
}

impl StartScreen {
    pub fn new(cell_dim: CellDim) -> Self {
        Self {
            control: Control::new(7.),
            cell_dim,

            palettes: vec![app::Palette::dark()],
            current_palette: 0,

            prefs: Prefs::default().apple_food(2),
            stats: Default::default(),

            player1_demo: SnakeDemo::new(HexPoint { h: 1, v: 5 }),
            player2_demo: SnakeDemo::new(HexPoint { h: 15, v: 5 }),

            rng: thread_rng(),
        }
    }
}

impl EventHandler<ggez::GameError> for StartScreen {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        while self.control.can_update() {
            let frame_stamp = self.control.frame_stamp();
            self.player1_demo.advance_snakes(frame_stamp, &self.prefs, &mut self.rng);
            self.player2_demo.advance_snakes(frame_stamp, &self.prefs, &mut self.rng);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.control.graphics_frame();
        let frame_stamp = self.control.frame_stamp();

        graphics::clear(ctx, Color::BLACK);

        let palette = &self.palettes[self.current_palette];
        self.player1_demo.draw(
            ctx,
            self.cell_dim,
            frame_stamp,
            self.prefs.draw_style,
            palette,
            &mut self.stats,
        )?;
        self.player2_demo.draw(
            ctx,
            self.cell_dim,
            frame_stamp,
            self.prefs.draw_style,
            palette,
            &mut self.stats,
        )?;

        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Left => self.player1_demo.next_palette(),
            KeyCode::Right => self.player2_demo.next_palette(),
            _ => (),
        }
    }
}

impl StartScreen {
    pub fn next_screen(&self) -> Option<Screen> {
        None
    }
}
