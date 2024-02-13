use crate::app::fps_control::FpsControl;
use crate::app::game_context::GameContext;
use crate::app::prefs::Prefs;
use crate::app::screen::Environment;
use crate::app::snake_management::{find_collisions, handle_collisions};
use crate::app::stats::Stats;
use crate::app::{self, Screen};
use crate::apple::spawn::{spawn_apples, SpawnPolicy, SpawnScheduleBuilder};
use crate::basic::{CellDim, Dir, HexPoint};
use crate::color::Color;
use crate::error::{Error, ErrorConversion, Result};
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::eat_mechanics::{EatBehavior, EatMechanics};
use crate::snake::SegmentType;
use crate::snake_control::Template;
use crate::view::snakes::OtherSnakes;
use crate::{apple, by_segment_type, by_snake_type, rendering, snake};
use ggez::event::EventHandler;
use ggez::graphics::{Canvas, DrawParam};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::Context;
use rand::prelude::*;
use std::cell::RefCell;
use std::default::Default;
use std::rc::Rc;
use std::result;

// position of the snake within the demo box is relative,
// the snake thinks it's in an absolute world at (0, 0)
struct SnakeDemo {
    location: HexPoint, // top-left
    env: Environment<NoRng>,

    palettes: Vec<snake::PaletteTemplate>,
    current_palette: usize,

    fps_control: Rc<RefCell<FpsControl>>,
}

impl SnakeDemo {
    fn new(
        cell_dim: CellDim,
        location: HexPoint,
        app_palette: app::Palette,
        control: Rc<RefCell<FpsControl>>,
    ) -> Self {
        let board_dim = HexPoint { h: 11, v: 8 };
        let start_pos = HexPoint { h: 4, v: 4 };
        let start_dir = Dir::U;
        let start_len = 10;

        let spawn_schedule = SpawnScheduleBuilder::new()
            .spawn(HexPoint { h: 5, v: 4 }, apple::Type::Food(1))
            .wait(40)
            .build();
        let apple_spawn_policy = SpawnPolicy::ScheduledOnEat {
            apple_count: 1,
            schedule: spawn_schedule,
            next_index: 0,
            current_wait: 0,
        };

        let snake_palettes = vec![
            snake::PaletteTemplate::rainbow(true),
            snake::PaletteTemplate::solid_white_red(),
            // snake::PaletteTemplate::rainbow(false),
            snake::PaletteTemplate::alternating_white(),
            snake::PaletteTemplate::gray_gradient(1., false),
            snake::PaletteTemplate::green_to_red(false),
            // snake::PaletteTemplate::zebra(),
        ];

        let seed = SnakeBuilder::default()
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::new(
                by_segment_type! {
                    SegmentType::DISCR_EATEN => EatBehavior::PassOver,
                    _ => EatBehavior::Cut,
                },
                by_snake_type! {
                    _ => by_segment_type! {
                        _ => EatBehavior::Crash,
                    }
                },
            ))
            // placeholder, updated immediately
            .palette(snake::PaletteTemplate::Solid {
                color: Color::RED,
                eaten: Color::RED,
            })
            .controller(Template::demo_8_pattern(0))
            .pos(start_pos)
            .dir(start_dir)
            .len(start_len)
            .speed(1.)
            .palette(snake_palettes[0]);

        Self {
            location,
            env: Environment {
                snakes: vec![seed.build().unwrap()],
                apples: vec![],
                gtx: GameContext::new(
                    board_dim,
                    cell_dim,
                    app_palette,
                    Prefs::default(),
                    apple_spawn_policy,
                ),
                rng: NoRng,
            },

            palettes: snake_palettes,
            current_palette: 0,

            fps_control: control,
        }
    }
}

impl SnakeDemo {
    fn prev_palette(&mut self) {
        self.current_palette =
            (self.current_palette + self.palettes.len() - 1) % self.palettes.len();
        self.env.snakes[0].palette = self.palettes[self.current_palette].into();
    }

    fn next_palette(&mut self) {
        self.current_palette = (self.current_palette + 1) % self.palettes.len();
        self.env.snakes[0].palette = self.palettes[self.current_palette].into();
    }

    fn update(&mut self, ctx: &Context) {
        // unimplemented!("how do you use GameContext here??")
        self.env.snakes[0].advance(
            OtherSnakes::empty(),
            &self.env.apples,
            &self.env.gtx,
            self.fps_control.borrow().context(),
            ctx,
        );

        let collisions = find_collisions(&self.env);
        let (spawn_snakes, game_over) = handle_collisions(&mut self.env, &collisions);

        assert!(spawn_snakes.is_empty(), "unexpected snake spawn");
        assert!(!game_over, "unexpected game over");

        spawn_apples(&mut self.env);
    }

    fn draw(&mut self, canvas: &mut Canvas, ctx: &mut Context, stats: &mut Stats) -> Result {
        self.env.snakes[0].update_dir(
            OtherSnakes::empty(),
            &[],
            &self.env.gtx,
            self.fps_control.borrow().context(),
            ctx,
        );

        let offset = self.location.to_cartesian(self.env.gtx.cell_dim);
        let draw_param = DrawParam::default().dest(offset);

        let grid_mesh = rendering::grid_mesh(&self.env.gtx, ctx)?;
        canvas.draw(&grid_mesh, draw_param);

        let border_mesh = rendering::border_mesh(&self.env.gtx, ctx)?;
        canvas.draw(&border_mesh, draw_param);

        let fps_control = self.fps_control.borrow();
        let ftx = fps_control.context();

        let snake_mesh =
            rendering::snake_mesh(&mut self.env.snakes, &self.env.gtx, ftx, ctx, stats)?;
        canvas.draw(&snake_mesh, draw_param);

        if !self.env.apples.is_empty() {
            let apple_mesh =
                rendering::apple_mesh(&self.env.apples, &self.env.gtx, ftx, ctx, stats)?;
            canvas.draw(&apple_mesh, draw_param);
        }

        let (button_mesh, clicked_left, clicked_right) =
            rendering::palette_changing_buttons_mesh(&self.env.gtx, ctx, offset)?;
        canvas.draw(&button_mesh, draw_param);

        drop(fps_control);

        assert!(!(clicked_left && clicked_right));
        if clicked_left {
            self.prev_palette();
        } else if clicked_right {
            self.next_palette();
        }

        Ok(())
    }
}

struct NoRng;

impl RngCore for NoRng {
    fn next_u32(&mut self) -> u32 {
        unimplemented!()
    }

    fn next_u64(&mut self) -> u64 {
        unimplemented!()
    }

    fn fill_bytes(&mut self, _dest: &mut [u8]) {
        unimplemented!()
    }

    fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> result::Result<(), rand::Error> {
        unimplemented!()
    }
}

pub struct StartScreen {
    fps_control: Rc<RefCell<FpsControl>>,

    // TODO: implement palette choice
    // palettes: Vec<app::Palette>,
    // current_palette: usize,
    palette: app::Palette,
    cell_dim: CellDim,

    player1_demo: SnakeDemo,
    player2_demo: SnakeDemo,

    stats: Stats,
}

impl StartScreen {
    #[allow(dead_code)]
    pub fn new(cell_dim: CellDim, app_palette: app::Palette) -> Self {
        let fps_control = Rc::new(RefCell::new(FpsControl::new(7.)));

        Self {
            fps_control: fps_control.clone(),

            palette: app_palette.clone(),
            cell_dim,

            player1_demo: SnakeDemo::new(
                cell_dim,
                HexPoint { h: 1, v: 5 },
                app_palette.clone(),
                fps_control.clone(),
            ),
            player2_demo: SnakeDemo::new(
                cell_dim,
                HexPoint { h: 15, v: 5 },
                app_palette,
                fps_control,
            ),

            stats: Default::default(),
        }
    }
}

impl EventHandler<Error> for StartScreen {
    fn update(&mut self, ctx: &mut Context) -> Result {
        while self.fps_control.borrow_mut().can_update() {
            self.player1_demo.update(ctx);
            self.player2_demo.update(ctx);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result {
        self.fps_control.borrow_mut().graphics_frame();

        let mut canvas = Canvas::from_frame(ctx, self.palette.background_color);

        self.player1_demo.draw(&mut canvas, ctx, &mut self.stats)?;
        self.player2_demo.draw(&mut canvas, ctx, &mut self.stats)?;

        let draw_param = DrawParam::default();
        let (button_mesh, _clicked_single, _clicked_double, message_single, message_double) =
            rendering::player_number_buttons_mesh(self.cell_dim, ctx)?;
        canvas.draw(&button_mesh, draw_param);
        if let Some(message) = message_single {
            message.draw(&mut canvas)
        }
        if let Some(message) = message_double {
            message.draw(&mut canvas)
        }

        canvas
            .finish(ctx)
            .map_err(Error::from)
            .with_trace_step("Game::draw")
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> Result {
        match input.keycode {
            Some(KeyCode::Left) => self.player1_demo.next_palette(),
            Some(KeyCode::Right) => self.player2_demo.next_palette(),
            _ => (),
        }
        Ok(())
    }
}

impl StartScreen {
    pub fn next_screen(&self) -> Option<Screen> {
        None
    }
}
