use std::collections::VecDeque;

use ggez::{event::EventHandler, graphics::{clear, draw, present, DrawParam}, Context, GameResult, GameError};

use crate::{
    app::{
        screen::{
            control::Control, prefs::Prefs, rendering::snake_mesh::get_snake_mesh, stats::Stats,
        },
        snake::{
            controller::{programmed::Move, ControllerTemplate},
            utils::OtherSnakes,
            Body, EatBehavior, EatMechanics, Segment, SegmentType, Snake, SnakeType, State,
        },
        Screen,
    },
    basic::{CellDim, Dir, HexDim, HexPoint},
};
use ggez::graphics::Color;
use std::slice;
use crate::app::snake::SnakeSeed;
use crate::app::screen::rendering::grid_mesh::{get_grid_mesh, get_border_mesh};
use crate::app;
use crate::app::screen::game::{FrameStamp, Apple, AppleType};
use crate::basic::DrawStyle;
use crate::app::snake;
use ggez::event::{MouseButton, KeyMods, Button, GamepadId, Axis, ErrorOrigin, KeyCode};
use crate::app::apple_spawn_strategy::{AppleSpawnStrategy, AppleSpawn};
use crate::app::screen::rendering::apple_mesh::get_apple_mesh;

// position of the snake within the demo box is relative,
// the snake thinks it's in an absolute world at (0, 0)
struct SnakeDemo {
    location: HexPoint, // top-left
    dim: HexDim,
    apples: Vec<Apple>,
    apple_spawn_strategy: AppleSpawnStrategy,
    snake: Snake,
    palettes: Vec<snake::PaletteTemplate>,
    current_palette: usize,
}

impl SnakeDemo {
    fn new(location: HexPoint) -> Self {
        let start_dir = Dir::U;
        let start_pos = HexPoint { h: 1, v: 4 };
        let board_dim = HexPoint { h: 11, v: 8 };

        let spawn_schedule = spawn_schedule![
            spawn(6, 2),
            wait(40),
        ];
        let apple_spawn_strategy = AppleSpawnStrategy::ScheduledOnEat {
            apple_count: 1,
            spawns: spawn_schedule,
            next_index: 0,
        };

        let mut seed = SnakeSeed {
            snake_type: SnakeType::Simulated {
                start_pos,
                start_dir,
                start_grow: 5,
            },
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
            controller: ControllerTemplate::demo_infinity_pattern(1),
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
            dim: board_dim,
            apples: vec![],
            apple_spawn_strategy,
            snake: Snake::from_seed(&seed, start_pos, start_dir, 5),
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

    fn spawn_apples(&mut self) {
        let apple = match &mut self.apple_spawn_strategy {
            AppleSpawnStrategy::ScheduledOnEat { apple_count, spawns, next_index } => {
                if self.apples.len() >= *apple_count {
                    return
                }

                let len = spawns.len();
                match &mut spawns[*next_index] {
                    AppleSpawn::Wait { total, current } => {
                        if *current == *total - 1 {
                            *current = 0;
                            *next_index = (*next_index + 1) % len;
                        } else {
                            *current += 1;
                        }
                        None
                    }
                    AppleSpawn::Spawn(pos) => {
                        *next_index = (*next_index + 1) % len;
                        Some(*pos)
                    }
                }
            }
            _ => unimplemented!(),
        };

        if let Some(pos) = apple {
            self.apples.push(Apple { pos, typ: AppleType::Normal(2) })
        }
    }

    fn advance_snakes(&mut self, frame_stamp: FrameStamp) {
        self.snake.advance(
            OtherSnakes::empty(),
            &self.apples,
            self.dim,
            frame_stamp,
        );


        // TODO: refactor snake cutting behavior out of StartScreen and Game
        let head_pos = self.snake.head().pos;
        let mut cut = None;
        for (segment_idx, segment) in self.snake.body.iter().enumerate().skip(1) {
            if segment.pos == head_pos {
                cut = Some(segment_idx);
                break;
            }
        }
        if let Some(idx) = cut {
            let _ = self.snake.body.cells.drain(idx..);
        }

        // TODO: refactor apple spawning and eating code out of StartScreen and Game
        // check apple eating
        for i in (0..self.apples.len()).rev() {
            if self.snake.head().pos == self.apples[i].pos {
                let Apple { typ, .. } = self.apples.remove(i);
                match typ {
                    AppleType::Normal(food) => {
                        self.snake.body.cells[0].typ = SegmentType::Eaten {
                            original_food: food,
                            food_left: food,
                        }
                    }
                    apple => panic!("unexpected apple {:?}", apple),
                }
            }
        }

        self.spawn_apples();
    }

    fn draw(&mut self, ctx: &mut Context, cell_dim: CellDim, frame_stamp: FrameStamp, draw_style: DrawStyle, palette: &app::Palette, stats: &mut Stats) -> GameResult {
        self.snake.update_dir(
            OtherSnakes::empty(),
            &[],
            self.dim,
            frame_stamp,
        );

        let draw_param = DrawParam::default().dest(self.location.to_cartesian(cell_dim));

        let grid_mesh = get_grid_mesh(self.dim, cell_dim, palette, ctx)?;
        draw(ctx, &grid_mesh, draw_param)?;

        let border_mesh = get_border_mesh(self.dim, cell_dim, palette, ctx)?;
        draw(ctx, &border_mesh, draw_param)?;

        let snake_mesh = get_snake_mesh(
            slice::from_mut(&mut self.snake),
            frame_stamp,
            self.dim,
            cell_dim,
            draw_style,
            ctx,
            stats,
        )?;
        draw(ctx, &snake_mesh, draw_param)?;

        if !self.apples.is_empty() {
            let apple_mesh = get_apple_mesh(&self.apples, frame_stamp, cell_dim, draw_style, palette, ctx, stats)?;
            draw(ctx, &apple_mesh, draw_param)?;
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
}

impl StartScreen {
    pub fn new(cell_dim: CellDim) -> Self {
        Self {
            control: Control::new(7.),
            cell_dim,
            palettes: vec![app::Palette::dark()],
            current_palette: 0,
            prefs: Default::default(),
            stats: Default::default(),
            player1_demo: SnakeDemo::new(HexPoint { h: 1, v: 5 }),
            player2_demo: SnakeDemo::new(HexPoint { h: 15, v: 5 }),
        }
    }
}

impl EventHandler<ggez::GameError> for StartScreen {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        while self.control.can_update() {
            let frame_stamp = self.control.frame_stamp();
            self.player1_demo.advance_snakes(frame_stamp);
            self.player2_demo.advance_snakes(frame_stamp);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.control.graphics_frame();
        let frame_stamp = self.control.frame_stamp();

        clear(ctx, Color::BLACK);

        let palette = &self.palettes[self.current_palette];
        self.player1_demo.draw(ctx, self.cell_dim, frame_stamp, self.prefs.draw_style, palette, &mut self.stats)?;
        self.player2_demo.draw(ctx, self.cell_dim, frame_stamp, self.prefs.draw_style, palette, &mut self.stats)?;

        present(ctx)
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
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
