use crate::game::{ctrl::Ctrl, snake::SnakeState};
use effect::Effect;
use ggez::{
    conf::WindowMode,
    event::{EventHandler, KeyMods},
    graphics::*,
    input::keyboard::KeyCode,
    Context, GameError, GameResult,
};
use hex::{HexPos, HexType::*};
use mint::Point2;
use num_integer::Integer;
use rand::prelude::*;
use snake::Snake;
use std::{f32::consts::PI, thread};
use theme::Theme;
use tuple::Map;

mod effect;
mod hex;
mod snake;
pub mod theme;

#[macro_use]
#[allow(dead_code)]
pub mod ctrl;

// todo explain this (cos_len < sin_len) (120deg angle was used)
#[derive(Copy, Clone)]
struct CellDim {
    side: f32,
    sin: f32,
    cos: f32,
}

// ggez frames per game frame
struct FramesPerFrame(u8, u8);

impl FramesPerFrame {
    fn new(fpf: u8) -> Self {
        Self(fpf, 0)
    }
    fn advance(&mut self) -> bool {
        self.1 += 1;
        if self.1 == self.0 {
            self.1 = 0;
            true
        } else {
            false
        }
    }
}

#[derive(Eq, PartialEq)]
enum GameState {
    Playing,
    Paused,
    Crashed,
}

pub struct Game {
    state: GameState,
    fpf: FramesPerFrame,

    dim: HexPos,
    players: Vec<Ctrl>,
    snakes: Vec<Snake>,
    apples: Vec<HexPos>,

    cell_dim: CellDim,
    theme: Theme,

    apple_count: usize,

    rng: ThreadRng,
    grid_mesh: Option<Mesh>,
    effect: Option<Effect>,
}

impl Game {
    pub fn new(cell_side_len: f32, players: Vec<Ctrl>, theme: Theme, wm: WindowMode) -> Self {
        assert!(!players.is_empty(), "No players specified");
        assert!(players.len() <= 2, "More than 2 players not supported");

        let (s, c) = (1. / 3. * PI).sin_cos().map(|i| i * cell_side_len);

        let h = wm.width / (cell_side_len + c);
        let v = wm.height / (2. * s);
        let dim = HexPos {
            h: h as isize,
            v: v as isize,
        };

        let mut game = Self {
            state: GameState::Playing,
            fpf: FramesPerFrame::new(5),

            dim,
            players,
            snakes: vec![],
            apples: vec![],

            cell_dim: CellDim {
                side: cell_side_len,
                sin: s,
                cos: c,
            },
            theme,

            apple_count: 50,

            rng: thread_rng(),
            grid_mesh: None,
            effect: None,
        };
        game.restart();
        game
    }

    // spawn 2 snakes in the middle
    fn restart(&mut self) {
        self.snakes.clear();
        self.apples.clear();

        // self.snakes.push(
        //     Snake::new(self.dim, HexPos { h: -2, v: 0 }, left_side_control));
        // self.snakes.push(
        //     Snake::new(self.dim, HexPos { h: 2, v: 0 }, right_side_control));
        // assert!()\
        // todo!()

        for (ctrl, &h) in self.players.iter().zip([-2, 2].iter()) {
            self.snakes
                .push(Snake::new(self.dim, HexPos { h, v: 0 }, ctrl.clone()))
        }
        self.spawn_apples();
        self.state = GameState::Playing;
    }

    // spawns apples until there are `total` apples in the game
    pub fn spawn_apples(&mut self) {
        'outer: while self.apples.len() < self.apple_count {
            let mut attempts = 0_u8;
            'apple_maker: loop {
                let apple_pos = HexPos::random_in(self.dim, &mut self.rng);
                attempts += 1;
                for s in self
                    .snakes
                    .iter()
                    .map(|s| s.body.iter().map(|b| b.pos))
                    .flatten()
                    .chain(self.apples.iter().copied())
                {
                    if s == apple_pos {
                        // make a new apple
                        assert!(attempts < 5);
                        continue 'apple_maker;
                    }
                }

                // apple made successfully
                self.apples.push(apple_pos);
                continue 'outer;
            }
        }
    }

    fn generate_grid_mesh(&self, ctx: &mut Context) -> GameResult<Mesh> {
        let CellDim {
            side: sl,
            sin: s,
            cos: c,
        } = self.cell_dim;

        let mut wall_points_a = vec![];
        let mut wall_points_b = vec![];

        let mut between_a_b = vec![];
        let mut between_b_a = vec![];

        for v in 0..self.dim.v {
            let dv = v as f32 * 2. * s;

            wall_points_a.push(Point2 { x: c, y: dv });
            wall_points_a.push(Point2 { x: 0., y: dv + s });

            wall_points_b.push(Point2 { x: c + sl, y: dv });
            wall_points_b.push(Point2 {
                x: 2. * c + sl,
                y: dv + s,
            });

            between_a_b.push(Point2 { x: c, y: dv });
            between_a_b.push(Point2 { x: c + sl, y: dv });

            between_b_a.push(Point2 {
                x: 2. * c + sl,
                y: dv + s,
            });
            between_b_a.push(Point2 {
                x: 2. * c + 2. * sl,
                y: dv + s,
            });
        }

        {
            let dv = self.dim.v as f32 * 2. * s;
            wall_points_a.push(Point2 { x: c, y: dv });
            wall_points_b.push(Point2 { x: c + sl, y: dv });
        }

        let mut builder = MeshBuilder::new();

        let draw_mode = DrawMode::stroke(self.theme.line_thickness);
        let fg = self.theme.palette.foreground_color;
        for h in 0..(self.dim.h + 1) / 2 {
            builder.polyline(draw_mode, &wall_points_a, fg)?;
            builder.polyline(draw_mode, &wall_points_b, fg)?;

            let dh = h as f32 * (2. * sl + 2. * c);

            for v in 0..self.dim.v {
                let dv = v as f32 * 2. * s;
                builder.line(
                    &[
                        Point2 { x: c + dh, y: dv },
                        Point2 {
                            x: c + sl + dh,
                            y: dv,
                        },
                    ],
                    2.,
                    fg,
                )?;

                builder.line(
                    &[
                        Point2 {
                            x: 2. * c + sl + dh,
                            y: s + dv,
                        },
                        Point2 {
                            x: 2. * c + 2. * sl + dh,
                            y: s + dv,
                        },
                    ],
                    2.,
                    fg,
                )?;
            }
            {
                let dv = self.dim.v as f32 * 2. * s;
                builder.line(
                    &[
                        Point2 { x: c + dh, y: dv },
                        Point2 {
                            x: c + sl + dh,
                            y: dv,
                        },
                    ],
                    2.,
                    fg,
                )?;
            }

            for (a, b) in wall_points_a.iter_mut().zip(&mut wall_points_b) {
                a.x += 2. * sl + 2. * c;
                b.x += 2. * sl + 2. * c;
            }
        }
        if self.dim.h.is_even() {
            builder.polyline(draw_mode, &wall_points_a[..wall_points_a.len() - 1], fg)?;
        }

        builder.build(ctx)
    }
}

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        if self.state != GameState::Playing {
            return Ok(());
        }

        if !self.fpf.advance() {
            return Ok(());
        }

        for snake in &mut self.snakes {
            snake.advance()
        }

        // check for crashes
        let mut crashed_snake_indices = vec![];
        'outer: for (i, snake) in self.snakes.iter().enumerate() {
            for (j, other) in self.snakes.iter().enumerate() {
                // check head-head crash
                if i != j && snake.body[0].pos == other.body[0].pos {
                    // snake j will be added when it's reached by the outer loop
                    crashed_snake_indices.push(i);
                    self.state = GameState::Crashed;
                    continue 'outer;
                }

                // check head-body crash (this also checks if a snake crashed with itself)
                for segment in &other.body[1..] {
                    if snake.body[0].pos == segment.pos {
                        crashed_snake_indices.push(i);
                        self.state = GameState::Crashed;
                        continue 'outer;
                    }
                }
            }
        }
        for i in crashed_snake_indices {
            self.snakes[i].state = SnakeState::Crashed;
            self.snakes[i].body[0].typ = Crashed;
        }

        // check apple eating
        for i in (0..self.apples.len()).rev() {
            for snake in &mut self.snakes {
                if snake.body[0].pos == self.apples[i] {
                    self.apples.remove(i);
                    snake.body[0].typ = Eaten(5);
                }
            }
        }

        self.spawn_apples();

        thread::yield_now();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        // note could be fun to implement optionally printing as a square grid
        // TODO: implement optional pushing to left-top (hiding part of the hexagon)
        // with 0, 0 the board is touching top-left (nothing hidden)

        clear(ctx, self.theme.palette.background_color);

        // shorter names look nice in formulas
        let CellDim {
            side: sl,
            sin: s,
            cos: c,
        } = self.cell_dim;

        // generate grid mesh first time, later reuse
        if self.grid_mesh.is_none() {
            self.grid_mesh = Some(self.generate_grid_mesh(ctx)?);
        }
        draw(ctx, self.grid_mesh.as_ref().unwrap(), DrawParam::default())?;

        let mut hexagon_points = [
            Point2 { x: c, y: 0. },
            Point2 { x: sl + c, y: 0. },
            Point2 {
                x: sl + 2. * c,
                y: s,
            },
            Point2 {
                x: sl + c,
                y: 2. * s,
            },
            Point2 { x: c, y: 2. * s },
            Point2 { x: 0., y: s },
            Point2 { x: c, y: 0. },
        ];

        // TODO: improve effect drawing and move it somewhere else
        if let Some(Effect::SmallHex {
            min_scale,
            iterations,
            passed,
        }) = self.effect
        {
            let scale_factor = if passed < iterations / 2 {
                let fraction = passed as f32 / (iterations as f32 / 2.);
                1. - fraction * (1. - min_scale)
            } else {
                let fraction =
                    (passed - iterations / 2) as f32 / (iterations - iterations / 2) as f32;
                1. - (1. - fraction) * (1. - min_scale)
            };

            // scale down and reposition in the middle of the hexagon
            for pt in &mut hexagon_points {
                pt.x *= scale_factor;
                pt.y *= scale_factor;
                // formula is (dimension / 2) * (1 - scale factor) [simplified]
                pt.x += (c + sl / 2.) * (1. - scale_factor);
                pt.y += s * (1. - scale_factor); // actually 2 * s / 2
            }

            if passed == iterations {
                self.effect = None;
            } else {
                // always succeeds, only used to unwrap
                if let Some(Effect::SmallHex { passed, .. }) = &mut self.effect {
                    *passed += 1
                }
            }
        }

        let mut builder = MeshBuilder::new();

        let mut draw_cell = |h: usize, v: usize, c: Color| {
            let offset_x = h as f32 * (self.cell_dim.side + self.cell_dim.cos);
            let offset_y =
                v as f32 * 2. * self.cell_dim.sin + if h % 2 == 0 { 0. } else { self.cell_dim.sin };

            let translated_hexagon = hexagon_points
                .iter()
                .map(|Point2 { x, y }| Point2 {
                    x: x + offset_x,
                    y: y + offset_y,
                })
                .collect::<Vec<_>>();
            builder
                .polyline(DrawMode::fill(), &translated_hexagon, c)
                .map(|_| ())
        };

        // draw snakes, crashed (collision) points on top
        for snake in &self.snakes {
            snake.draw_non_crash_points(&mut draw_cell, &self.theme.palette)?;
        }

        for snake in &self.snakes {
            snake.draw_crash_point(&mut draw_cell, &self.theme.palette)?;
        }

        for apple in &self.apples {
            draw_cell(
                apple.h as usize,
                apple.v as usize,
                self.theme.palette.apple_fill_color,
            )?
        }

        let mesh = &builder.build(ctx)?;
        draw(ctx, mesh, DrawParam::default())?;

        thread::yield_now();
        present(ctx)
    }

    fn key_down_event(&mut self, _ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
        if key == KeyCode::Space {
            use GameState::*;
            match self.state {
                Crashed => self.restart(),
                Playing => self.state = Paused,
                Paused => self.state = Playing,
            }
        } else {
            for snake in &mut self.snakes {
                snake.key_pressed(key)
            }
        }
    }

    // broken (may be a mac problem)
    // fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
    //     println!("WARNING: resize broken");
    //     return;
    //
    //     let dim = wh_to_dim(width, height, self.cell_side_len);
    //     self.dim = dim;
    //     for snake in &mut self.snakes {
    //         snake.game_dim = dim;
    //     }
    //
    //     let new_wm = WindowMode {
    //         width,
    //         height,
    //         maximized: false,
    //         fullscreen_type: FullscreenType::Windowed,
    //         borderless: false,
    //         min_width: 0.,
    //         min_height: 0.,
    //         max_width: 0.,
    //         max_height: 0.,
    //         resizable: true,
    //     };
    //     set_mode(ctx, new_wm)
    //         .expect("failed to resize window");
    //
    //     // println!("w/h: {}/{}", width, height);
    //     // self.effect = Some(Effect::SmallHex {
    //     //     min_scale: 0.1,
    //     //     iterations: 10,
    //     //     passed: 0,
    //     // });
    // }
}
