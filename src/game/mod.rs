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
use crate::game::hex::Hex;
use crate::game::snake::Dir;
use ggez::conf::FullscreenType;

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

struct HexagonPoints {
    full: [Point2<f32>; 6],
    u: [Point2<f32>; 4],
    d: [Point2<f32>; 4],
    ul: [Point2<f32>; 4],
    ur: [Point2<f32>; 4],
    dl: [Point2<f32>; 4],
    dr: [Point2<f32>; 4],
}

#[derive(Eq, PartialEq)]
enum GameState {
    Playing,
    Paused,
    Crashed,
}

struct Prefs {
    draw_grid: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            draw_grid: true,
        }
    }
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
    hexagon_points: HexagonPoints,
    grid_mesh: Option<Mesh>,
    effect: Option<Effect>,

    prefs: Prefs,
}

impl Game {
    fn wh_to_dim(cell_dim: CellDim, width: f32, height: f32) -> HexPos {
        let CellDim { side, sin, cos } = cell_dim;
        HexPos {
            h: (width / (side + cos)) as isize,
            v: (height / (2. * sin)) as isize - 1,
        }
    }

    pub fn new(cell_side_len: f32, players: Vec<Ctrl>, theme: Theme, wm: WindowMode) -> Self {
        assert!(!players.is_empty(), "No players specified");
        assert!(players.len() <= 2, "More than 2 players not supported");

        let side = cell_side_len;
        let (sin, cos) = (1. / 3. * PI).sin_cos().map(|i| i * side);
        let cell_dim = CellDim { side, sin, cos };

        let hexagon_points = HexagonPoints {
            full: [
                Point2 { x: cos, y: 0. },
                Point2 { x: side + cos, y: 0. },
                Point2 { x: side + 2. * cos, y: sin },
                Point2 { x: side + cos, y: 2. * sin },
                Point2 { x: cos, y: 2. * sin },
                Point2 { x: 0., y: sin },
            ],
            u: [
                Point2 { x: cos, y: 0. },
                Point2 { x: side + cos, y: 0. },
                Point2 { x: side + 2. * cos, y: sin },
                Point2 { x: 0., y: sin },
            ],
            d: [
                Point2 { x: side + 2. * cos, y: sin },
                Point2 { x: side + cos, y: 2. * sin },
                Point2 { x: cos, y: 2. * sin },
                Point2 { x: 0., y: sin },
            ],
            ul: [
                Point2 { x: cos, y: 0. },
                Point2 { x: side + cos, y: 0. },
                Point2 { x: cos, y: 2. * sin },
                Point2 { x: 0., y: sin },
            ],
            ur: [
                Point2 { x: cos, y: 0. },
                Point2 { x: side + cos, y: 0. },
                Point2 { x: side + 2. * cos, y: sin },
                Point2 { x: side + cos, y: 2. * sin },
            ],
            dl: [
                Point2 { x: cos, y: 0. },
                Point2 { x: side + cos, y: 2. * sin },
                Point2 { x: cos, y: 2. * sin },
                Point2 { x: 0., y: sin },
            ],
            dr: [
                Point2 { x: side + cos, y: 0. },
                Point2 { x: side + 2. * cos, y: sin },
                Point2 { x: side + cos, y: 2. * sin },
                Point2 { x: cos, y: 2. * sin },
            ],
        };

        let mut game = Self {
            state: GameState::Playing,
            fpf: FramesPerFrame::new(5),

            dim: Self::wh_to_dim(cell_dim, wm.width, wm.height),
            players,
            snakes: vec![],
            apples: vec![],

            cell_dim,
            theme,

            apple_count: 5,

            rng: thread_rng(),
            hexagon_points,
            grid_mesh: None,
            effect: None,

            prefs: Prefs::default(),
        };
        game.restart();
        game
    }

    // spawn 2 snakes in the middle
    fn restart(&mut self) {
        self.snakes.clear();
        self.apples.clear();

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
        let CellDim { side, sin, cos } = self.cell_dim;

        // two kinds of alternating vertical lines
        let mut vline_a = vec![];
        let mut vline_b = vec![];

        for dv in (0..=self.dim.v).map(|v| v as f32 * 2. * sin) {
            vline_a.push(Point2 { x: cos, y: dv });
            vline_a.push(Point2 { x: 0., y: dv + sin });

            vline_b.push(Point2 { x: cos + side, y: dv });
            vline_b.push(Point2 { x: 2. * cos + side, y: dv + sin });
        }

        let mut builder = MeshBuilder::new();

        let draw_mode = DrawMode::stroke(self.theme.line_thickness);
        let fg = self.theme.palette.foreground_color;
        for h in 0..(self.dim.h + 1) / 2 {
            if h == 0 {
                builder.polyline(draw_mode, &vline_a[..vline_a.len() - 1], fg)?;
            } else {
                builder.polyline(draw_mode, &vline_a, fg)?;
            }
            if self.dim.h.is_odd() && h == (self.dim.h + 1) / 2 - 1 {
                builder.polyline(draw_mode, &vline_b[..vline_b.len() - 1], fg)?;
            } else {
                builder.polyline(draw_mode, &vline_b, fg)?;
            }

            let dh = h as f32 * (2. * side + 2. * cos);

            for v in 0..=self.dim.v {
                let dv = v as f32 * 2. * sin;

                // line between a and b
                builder.line(
                    &[
                        Point2 { x: cos + dh, y: dv },
                        Point2 { x: cos + side + dh, y: dv },
                    ], 2., fg,
                )?;

                // line between b and a
                if !(self.dim.h.is_odd() && h == (self.dim.h + 1) / 2 - 1) {
                    builder.line(
                        &[
                            Point2 { x: 2. * cos + side + dh, y: sin + dv },
                            Point2 { x: 2. * cos + 2. * side + dh, y: sin + dv },
                        ], 2., fg,
                    )?;
                }
            }

            // shift the lines right by 2 cells
            for (a, b) in vline_a.iter_mut().zip(&mut vline_b) {
                a.x += 2. * (side + cos);
                b.x += 2. * (side + cos);
            }
        }
        if self.dim.h.is_even() {
            builder.polyline(draw_mode, &vline_a[1..], fg)?;
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
        // NOTE: could be fun to implement optionally printing as a square grid
        // TODO: implement optional pushing to left-top (hiding part of the hexagon)
        // with 0, 0 the board is touching top-left (nothing hidden)

        clear(ctx, self.theme.palette.background_color);

        if self.prefs.draw_grid {
            // generate grid mesh first time, later reuse
            if self.grid_mesh.is_none() {
                self.grid_mesh = Some(self.generate_grid_mesh(ctx)?);
            }
            draw(ctx, self.grid_mesh.as_ref().unwrap(), DrawParam::default())?;
        }

        // TODO: improve effect drawing and move it somewhere else
        // if let Some(Effect::SmallHex {
        //                 min_scale,
        //                 iterations,
        //                 passed,
        //             }) = self.effect
        // {
        //     let scale_factor = if passed < iterations / 2 {
        //         let fraction = passed as f32 / (iterations as f32 / 2.);
        //         1. - fraction * (1. - min_scale)
        //     } else {
        //         let fraction =
        //             (passed - iterations / 2) as f32 / (iterations - iterations / 2) as f32;
        //         1. - (1. - fraction) * (1. - min_scale)
        //     };
        //
        //     // scale down and reposition in the middle of the hexagon
        //     for pt in &mut hexagon_points {
        //         pt.x *= scale_factor;
        //         pt.y *= scale_factor;
        //         // formula is (dimension / 2) * (1 - scale factor) [simplified]
        //         pt.x += (cos + side / 2.) * (1. - scale_factor);
        //         pt.y += sin * (1. - scale_factor); // actually 2 * s / 2
        //     }
        //
        //     if passed == iterations {
        //         self.effect = None;
        //     } else {
        //         // always succeeds, only used to unwrap
        //         if let Some(Effect::SmallHex { passed, .. }) = &mut self.effect {
        //             *passed += 1
        //         }
        //     }
        // }

        let mut builder = MeshBuilder::new();

        let mut draw_cell = |h: usize, v: usize, c: Color, dir: Option<Dir>| {
            let offset_x = h as f32 * (self.cell_dim.side + self.cell_dim.cos);
            let offset_y =
                v as f32 * 2. * self.cell_dim.sin + if h % 2 == 0 { 0. } else { self.cell_dim.sin };

            use Dir::*;
            let points: &[_] = match dir {
                None => &self.hexagon_points.full,
                Some(U) => &self.hexagon_points.u,
                Some(D) => &self.hexagon_points.d,
                Some(UL) => &self.hexagon_points.ul,
                Some(UR) => &self.hexagon_points.ur,
                Some(DL) => &self.hexagon_points.dl,
                Some(DR) => &self.hexagon_points.dr,
            };

            let translated_points = points
                .iter()
                .map(|Point2 { x, y }| Point2 {
                    x: x + offset_x,
                    y: y + offset_y,
                })
                .collect::<Vec<_>>();
            builder
                .polyline(DrawMode::fill(), &translated_points, c)
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
                None,
            )?
        }

        let mesh = &builder.build(ctx)?;
        draw(ctx, mesh, DrawParam::default())?;

        thread::yield_now();
        present(ctx)
    }

    fn key_down_event(&mut self, _ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
        use GameState::*;
        use KeyCode::*;
        match key {
            Space => match self.state {
                Crashed => self.restart(),
                Playing => self.state = Paused,
                Paused => self.state = Playing,
            },
            G => self.prefs.draw_grid = !self.prefs.draw_grid,
            k => if self.state == Playing {
                for snake in &mut self.snakes {
                    snake.key_pressed(k)
                }
            }
        }
    }

    // TODO: forbid resizing in-game
    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        ggez::graphics::set_screen_coordinates(ctx, Rect {
            x: 0.0,
            y: 0.0,
            w: width,
            h: height,
        }).unwrap();

        let dim = Self::wh_to_dim(self.cell_dim, width, height);
        self.dim = dim;
        for snake in &mut self.snakes {
            snake.game_dim = dim;
        }
        self.grid_mesh = None;
    }
}
