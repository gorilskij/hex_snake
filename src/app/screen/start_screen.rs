use std::slice;

use crate::app::fps_control::FpsControl;
use crate::app::game_context::GameContext;
use crate::app::prefs::Prefs;
use crate::app::screen::Environment;
use crate::app::stats::Stats;
use crate::app::Screen;
use crate::apple::spawn::SpawnPolicy;
use crate::apple::Apple;
use crate::basic::{CellDim, Dir, FrameStamp, HexDim, HexPoint};
use crate::color::Color;
use crate::error::{Error, Result};
use crate::snake::{self, EatBehavior, EatMechanics, Snake};
use crate::snake_control::Template;
use crate::{app, rendering, spawn_schedule};
use ggez::event::EventHandler;
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::Context;
use rand::prelude::*;
use rand::rngs::ThreadRng;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::result;

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

    fps_control: Weak<RefCell<FpsControl>>,
}

impl SnakeDemo {
    fn new(location: HexPoint, control: Weak<RefCell<FpsControl>>) -> Self {
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

        let palettes = vec![
            snake::PaletteTemplate::solid_white_red(),
            // snake::PaletteTemplate::rainbow(false),
            snake::PaletteTemplate::rainbow(true),
            snake::PaletteTemplate::alternating_white(),
            snake::PaletteTemplate::gray_gradient(1., false),
            snake::PaletteTemplate::green_to_red(false),
            // snake::PaletteTemplate::zebra(),
        ];

        let seed = snake::Builder::default()
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::always(EatBehavior::Cut))
            // placeholder, updated immediately
            .palette(snake::PaletteTemplate::Solid {
                color: Color::RED,
                eaten: Color::RED,
            })
            .controller(Template::demo_infinity_pattern(1))
            .pos(start_pos)
            .dir(start_dir)
            .len(start_len)
            .palette(palettes[0]);

        Self {
            location,
            board_dim,
            apples: vec![],
            apple_spawn_policy,
            snake: seed.build().unwrap(),
            palettes,
            current_palette: 0,

            fps_control: control,
        }
    }
}

impl SnakeDemo {
    fn next_palette(&mut self) {
        self.current_palette = (self.current_palette + 1) % self.palettes.len();
        self.snake.palette = self.palettes[self.current_palette].into();
    }

    fn spawn_apples(&mut self, _prefs: &Prefs, _rng: &mut impl Rng) {
        unimplemented!("how do you use GameContext here??")
        // let new_apples = spawn_apples(
        //     slice::from_ref(&self.snake),
        //     &self.apples,
        //     &self.gtx,
        //     rng,
        // );
        // self.apples.extend(new_apples.into_iter());
    }

    fn advance_snakes(
        &mut self,
        _cell_dim: CellDim,
        _frame_stamp: FrameStamp,
        _prefs: &Prefs,
        _ctx: &Context,
        _rng: &mut impl Rng,
    ) {
        unimplemented!("how do you use GameContext here??")
        // self.snake.advance(
        //     OtherSnakes::empty(),
        //     &self.apples,
        //     self.board_dim,
        //     cell_dim,
        //     ctx,
        //     frame_stamp,
        // );

        // let collisions = find_collisions(self);
        // let (spawn_snakes, game_over) = handle_collisions(self, &collisions);
        //
        // assert!(spawn_snakes.is_empty(), "unexpected snake spawn");
        // assert!(!game_over, "unexpected game over");
        //
        // self.spawn_apples(prefs, rng);
    }

    fn draw(
        &mut self,
        _ctx: &mut Context,
        _cell_dim: CellDim,
        _frame_stamp: FrameStamp,
        _draw_style: rendering::Style,
        _palette: &app::Palette,
        _stats: &mut Stats,
    ) -> Result {
        unimplemented!("how do you use GameContext here??")
        // self.snake
        //     .update_dir(OtherSnakes::empty(), &[], self.board_dim, cell_dim, ctx, frame_stamp);
        //
        // let draw_param = DrawParam::default().dest(self.location.to_cartesian(cell_dim));
        //
        // let grid_mesh = rendering::grid_mesh(self.board_dim, cell_dim, palette, ctx)?;
        // graphics::draw(ctx, &grid_mesh, draw_param)?;
        //
        // let border_mesh = rendering::border_mesh(self.board_dim, cell_dim, palette, ctx)?;
        // graphics::draw(ctx, &border_mesh, draw_param)?;
        //
        // let snake_mesh = rendering::snake_mesh(
        //     slice::from_mut(&mut self.snake),
        //     frame_stamp,
        //     self.board_dim,
        //     cell_dim,
        //     draw_style,
        //     ctx,
        //     stats,
        // )?;
        // graphics::draw(ctx, &snake_mesh, draw_param)?;
        //
        // if !self.apples.is_empty() {
        //     let apple_mesh = rendering::apple_mesh(
        //         &self.apples,
        //         frame_stamp,
        //         cell_dim,
        //         draw_style,
        //         palette,
        //         ctx,
        //         stats,
        //     )?;
        //     graphics::draw(ctx, &apple_mesh, draw_param)?;
        // }
        //
        // Ok(())
    }
}

enum NoRng {}

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

impl Environment<NoRng> for SnakeDemo {
    fn snakes(&self) -> &[Snake] {
        slice::from_ref(&self.snake)
    }

    fn apples(&self) -> &[Apple] {
        &self.apples
    }

    fn snakes_apples_gtx_mut(&mut self) -> (&mut [Snake], &mut [Apple], &mut GameContext) {
        unimplemented!()
        // (slice::from_mut(&mut self.snake), &mut self.apples)
    }

    fn snakes_apples_rng_mut(&mut self) -> (&mut [Snake], &mut [Apple], &mut NoRng) {
        panic!("tried to get rng of SnakeDemo")
    }

    fn add_snake(&mut self, snake_builder: &snake::Builder) -> Result {
        panic!("tried to add snake to SnakeDemo: {snake_builder:?}")
    }

    fn remove_snake(&mut self, index: usize) -> Snake {
        panic!("tried to remove snake at index {index} in SnakeDemo")
    }

    fn remove_apple(&mut self, index: usize) -> Apple {
        self.apples.remove(index)
    }

    fn gtx(&self) -> &GameContext {
        panic!("SnakeDemo has no GameContext")
    }

    fn board_dim(&self) -> HexDim {
        self.board_dim
    }

    fn cell_dim(&self) -> CellDim {
        panic!("tried to get cell_dim from StartScreen")
    }

    fn frame_stamp(&self) -> FrameStamp {
        self.fps_control
            .upgrade()
            .expect("Weak pointer dropped")
            .borrow()
            .frame_stamp()
    }

    fn rng(&mut self) -> &mut NoRng {
        panic!("tried to get rng of SnakeDemo")
    }
}

pub struct StartScreen {
    fps_control: Rc<RefCell<FpsControl>>,
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
    #[allow(dead_code)]
    pub fn new(cell_dim: CellDim) -> Self {
        let fps_control = Rc::new(RefCell::new(FpsControl::new(7.)));
        let weak1 = Rc::downgrade(&fps_control);
        let weak2 = Rc::downgrade(&fps_control);

        Self {
            fps_control,
            cell_dim,

            palettes: vec![app::Palette::dark()],
            current_palette: 0,

            prefs: Prefs::default().apple_food(2),
            stats: Default::default(),

            player1_demo: SnakeDemo::new(HexPoint { h: 1, v: 5 }, weak1),
            player2_demo: SnakeDemo::new(HexPoint { h: 15, v: 5 }, weak2),

            rng: thread_rng(),
        }
    }
}

impl EventHandler<Error> for StartScreen {
    fn update(&mut self, _ctx: &mut Context) -> Result {
        unimplemented!("how do you use GameContext here??")
        // while self.control.borrow_mut().can_update(&mut self.gtx) {
        //     let frame_stamp = self.control.borrow().frame_stamp();
        //     self.player1_demo.advance_snakes(
        //         self.cell_dim,
        //         frame_stamp,
        //         &self.prefs,
        //         ctx,
        //         &mut self.rng,
        //     );
        //     self.player2_demo.advance_snakes(
        //         self.cell_dim,
        //         frame_stamp,
        //         &self.prefs,
        //         ctx,
        //         &mut self.rng,
        //     );
        // }
        // Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> Result {
        unimplemented!("how do you use GameContext here??")
        // self.control.borrow_mut().graphics_frame(&mut self.gtx);
        // let frame_stamp = self.control.borrow().frame_stamp();
        //
        // graphics::clear(ctx, Color::BLACK);
        //
        // let palette = &self.palettes[self.current_palette];
        // self.player1_demo.draw(
        //     ctx,
        //     self.cell_dim,
        //     frame_stamp,
        //     self.prefs.draw_style,
        //     palette,
        //     &mut self.stats,
        // )?;
        // self.player2_demo.draw(
        //     ctx,
        //     self.cell_dim,
        //     frame_stamp,
        //     self.prefs.draw_style,
        //     palette,
        //     &mut self.stats,
        // )?;
        //
        // graphics::present(ctx)
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
