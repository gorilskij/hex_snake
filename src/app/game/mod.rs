use std::{
    cmp::min,
    f32::consts::PI,
    io::{stdout, Write},
    thread,
    time::{Duration, Instant},
};

use ggez::{
    conf::WindowMode,
    event::{EventHandler, KeyCode, KeyMods},
    graphics::{
        clear, draw, present, Color, DrawMode, DrawParam, Font, Mesh, MeshBuilder, Scale, Text,
    },
    Context, GameResult,
};
use itertools::Itertools;
use mint::Point2;
use num_integer::Integer;
use rand::prelude::*;

use crate::app::{
    hex::{Dir, Hex, HexPos, HexType},
    palette::GamePalette,
    snake::{build_cell, Snake, SnakeSeed, SnakeState},
};
use std::collections::VecDeque;

// TODO document
#[derive(Copy, Clone)]
pub struct CellDim {
    pub side: f32,
    pub sin: f32,
    pub cos: f32,
}

impl From<f32> for CellDim {
    fn from(side: f32) -> Self {
        let one_third_pi = 1. / 3. * PI;
        Self {
            side,
            sin: one_third_pi.sin() * side,
            cos: one_third_pi.cos() * side,
        }
    }
}

struct FPSControl {
    frame_duration: Duration,   // for update and draw
    control_duration: Duration, // for key events (more frequent)
    last_frame: Option<Instant>,
    drawn: bool,
}

impl FPSControl {
    fn new(update_fps: u64, control_fps: u64) -> Self {
        Self {
            frame_duration: Duration::from_micros(1_000_000 / update_fps),
            control_duration: Duration::from_micros(1_000_000 / control_fps),
            // last_frame: Instant::now(),
            last_frame: None,
            drawn: false,
        }
    }

    fn maybe_update(&mut self) -> bool {
        // NOTE: this keeps being called when the game is paused
        // maybe implement a more relaxed control fps for pause
        let last_frame = self.last_frame.get_or_insert(Instant::now());
        let can_update = last_frame.elapsed() >= self.frame_duration;
        if can_update {
            *last_frame += self.frame_duration;
            self.drawn = false;
        }
        can_update
    }

    fn maybe_draw(&mut self) -> bool {
        let can_draw = !self.drawn;
        self.drawn = true;
        can_draw
    }

    fn wait(&mut self) {
        let last_frame = self.last_frame.get_or_insert(Instant::now());
        let frame_wait = self.frame_duration.checked_sub(last_frame.elapsed());
        let wait = frame_wait.map(|fw| min(fw, self.control_duration));
        if let Some(w) = wait {
            thread::sleep(w);
        }
    }
}

type HexagonPoints = [Point2<f32>; 6];

pub fn hexagon_points(side: f32) -> HexagonPoints {
    static mut CACHED_HEXAGON_POINTS: Option<(f32, HexagonPoints)> = None;

    unsafe {
        if let Some((cached_side, points)) = CACHED_HEXAGON_POINTS {
            if (side - cached_side).abs() < f32::EPSILON {
                return points;
            }
        }

        let CellDim { sin, cos, .. } = CellDim::from(side);
        #[rustfmt::skip]
        let points = [
            Point2 { x: cos, y: 0. },
            Point2 { x: side + cos, y: 0., },
            Point2 { x: side + 2. * cos, y: sin, },
            Point2 { x: side + cos, y: 2. * sin, },
            Point2 { x: cos, y: 2. * sin, },
            Point2 { x: 0., y: sin },
        ];

        CACHED_HEXAGON_POINTS = Some((side, points));
        points
    }
}

#[derive(Eq, PartialEq)]
enum GameState {
    Playing,
    Paused,
    Crashed,
}

// Mixed is Cut when eating self and Die when eating others
#[derive(Eq, PartialEq)]
enum EatBehavior {
    Cut,
    Die,
    Mixed,
}

struct Prefs {
    draw_grid: bool,
    eat_behavior: EatBehavior,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            draw_grid: true,
            eat_behavior: EatBehavior::Mixed,
        }
    }
}

struct Message(String, u8);

pub struct Game {
    state: GameState,
    fps: FPSControl,

    dim: HexPos,
    players: Vec<SnakeSeed>,
    snakes: Vec<Snake>,
    apples: Vec<HexPos>,

    cell_dim: CellDim,
    palette: GamePalette,

    apple_count: usize,

    rng: ThreadRng,
    grid_mesh: Option<Mesh>,
    border_mesh: Option<Mesh>,

    prefs: Prefs,
    message: Option<Message>,

    force_redraw: usize, // for a number of redraws, even when paused
}

impl Game {
    fn wh_to_dim(cell_dim: CellDim, width: f32, height: f32) -> HexPos {
        let CellDim { side, sin, cos } = cell_dim;
        HexPos {
            h: (width / (side + cos)) as isize,
            v: (height / (2. * sin)) as isize - 1,
        }
    }

    pub fn new(
        cell_side_len: f32,
        players: Vec<SnakeSeed>,
        palette: GamePalette,
        wm: WindowMode,
    ) -> Self {
        assert!(!players.is_empty(), "No players specified");

        let cell_dim = CellDim::from(cell_side_len);

        let mut game = Self {
            state: GameState::Playing,
            fps: FPSControl::new(12, 60),
            // fps: FPSControl::new(240, 240),
            dim: Self::wh_to_dim(cell_dim, wm.width, wm.height),
            players: players.into_iter().map(Into::into).collect(),
            snakes: vec![],
            apples: vec![],

            cell_dim,
            palette,

            apple_count: 5,

            rng: thread_rng(),
            grid_mesh: None,
            border_mesh: None,

            prefs: Prefs::default(),
            message: None,

            force_redraw: 0,
        };
        game.restart();
        game
    }

    fn restart(&mut self) {
        self.snakes.clear();
        self.apples.clear();

        const DISTANCE_BETWEEN_SNAKES: isize = 10;

        let total_width = (self.players.len() - 1) as isize * DISTANCE_BETWEEN_SNAKES + 1;
        assert!(total_width < self.dim.h, "snakes spread too wide");

        let half = total_width / 2;
        let middle = self.dim.h / 2;
        let start = middle - half;
        let end = start + total_width;
        let mut dir = Dir::U;
        for (seed, h_pos) in self
            .players
            .iter()
            .zip((start..=end).step_by(DISTANCE_BETWEEN_SNAKES as usize))
        {
            self.snakes.push(Snake::from_seed(
                seed,
                HexPos {
                    h: h_pos,
                    v: self.dim.v / 2,
                },
                dir,
                10,
                self.dim,
            ));

            // alternate
            dir = -dir;
        }

        self.spawn_apples();
        self.state = GameState::Playing;
    }

    pub fn occupied_cells(&self) -> Vec<HexPos> {
        // upper bound
        let max_occupied_cells = self
            .snakes
            .iter()
            .map(|snake| snake.body.len())
            .sum::<usize>()
            + self.apples.len();
        let mut occupied_cells = Vec::with_capacity(max_occupied_cells);
        occupied_cells.extend_from_slice(&self.apples);
        for snake in &self.snakes {
            occupied_cells.extend(snake.body.iter().map(|hex| hex.pos));
        }
        occupied_cells.sort_by_key(move |&x| x.v * self.dim.h + x.h);
        occupied_cells.dedup();
        occupied_cells
    }

    pub fn spawn_apples(&mut self) {
        let mut occupied_cells = self.occupied_cells();

        while self.apples.len() < self.apple_count {
            let free_spaces = (self.dim.h * self.dim.v) as usize - occupied_cells.len();
            if free_spaces == 0 {
                println!(
                    "warning: no space left for new apples ({} apples will be missing)",
                    self.apple_count - self.apples.len()
                );
                return;
            }
            let mut new_idx = self.rng.gen_range(0, free_spaces);
            for HexPos { h, v } in &occupied_cells {
                let idx = (v * self.dim.h + h) as usize;
                if idx <= new_idx {
                    new_idx += 1;
                }
            }
            assert!(new_idx < (self.dim.h * self.dim.v) as usize);
            let new_apple = HexPos {
                h: new_idx as isize % self.dim.h,
                v: new_idx as isize / self.dim.h,
            };
            // insert at sorted position
            match occupied_cells.binary_search(&new_apple) {
                Ok(idx) => panic!("Spawned apple at occupied cell {:?}", occupied_cells[idx]),
                Err(idx) => occupied_cells.insert(idx, new_apple),
            }
            self.apples.push(new_apple);
        }
    }

    fn generate_meshes(&self, ctx: &mut Context) -> GameResult<(Mesh, Mesh)> {
        let CellDim { side, sin, cos } = self.cell_dim;

        // two kinds of alternating vertical lines
        let mut vline_a = vec![];
        let mut vline_b = vec![];

        for dv in (0..=self.dim.v).map(|v| v as f32 * 2. * sin) {
            vline_a.push(Point2 { x: cos, y: dv });
            vline_a.push(Point2 { x: 0., y: dv + sin });

            vline_b.push(Point2 {
                x: cos + side,
                y: dv,
            });
            vline_b.push(Point2 {
                x: 2. * cos + side,
                y: dv + sin,
            });
        }

        let first_vline_a = vline_a.iter().copied().collect::<Vec<_>>();
        let mut last_vline_b = vec![];

        let grid_mesh = {
            let mut builder = MeshBuilder::new();

            let draw_mode = DrawMode::stroke(self.palette.grid_thickness);
            let color = self.palette.grid_color;
            for h in 0..(self.dim.h + 1) / 2 {
                if h == 0 {
                    builder.polyline(draw_mode, &vline_a[..vline_a.len() - 1], color)?;
                } else {
                    builder.polyline(draw_mode, &vline_a, color)?;
                }
                if self.dim.h.is_odd() && h == (self.dim.h + 1) / 2 - 1 {
                    builder.polyline(draw_mode, &vline_b[..vline_b.len() - 1], color)?;
                } else {
                    builder.polyline(draw_mode, &vline_b, color)?;
                }

                if h == (self.dim.h + 1) / 2 - 1 {
                    if self.dim.h.is_odd() {
                        last_vline_b = vline_b[..vline_b.len() - 1].iter().copied().collect();
                    } else {
                        last_vline_b = vline_b.iter().copied().collect();
                    }
                }

                let dh = h as f32 * (2. * side + 2. * cos);

                for v in 0..=self.dim.v {
                    let dv = v as f32 * 2. * sin;

                    // line between a and b
                    builder.line(
                        &[
                            Point2 { x: cos + dh, y: dv },
                            Point2 {
                                x: cos + side + dh,
                                y: dv,
                            },
                        ],
                        self.palette.grid_thickness,
                        color,
                    )?;

                    // line between b and a
                    if !(self.dim.h.is_odd() && h == (self.dim.h + 1) / 2 - 1) {
                        builder.line(
                            &[
                                Point2 {
                                    x: 2. * cos + side + dh,
                                    y: sin + dv,
                                },
                                Point2 {
                                    x: 2. * cos + 2. * side + dh,
                                    y: sin + dv,
                                },
                            ],
                            self.palette.grid_thickness,
                            color,
                        )?;
                    }
                }

                // shift the lines right by 2 cells
                let offset = 2. * (side + cos);
                vline_a.iter_mut().for_each(|a| a.x += offset);
                vline_b.iter_mut().for_each(|b| b.x += offset);
            }
            if self.dim.h.is_even() {
                builder.polyline(draw_mode, &vline_a[1..], color)?;
            }

            builder.build(ctx)?
        };

        assert!(!last_vline_b.is_empty());

        let border_mesh = {
            let mut builder = MeshBuilder::new();

            let draw_mode = DrawMode::stroke(self.palette.border_thickness);
            let color = self.palette.border_color;
            builder.polyline(draw_mode, &first_vline_a[..first_vline_a.len() - 1], color)?;
            builder.polyline(draw_mode, &last_vline_b, color)?;

            builder.build(ctx)?
        };

        Ok((grid_mesh, border_mesh))
    }
}

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // it's important that this if go first to reset the last frame time
        if !self.fps.maybe_update() {
            self.fps.wait();
            return Ok(());
        }

        if self.state != GameState::Playing {
            self.fps.wait();
            return Ok(());
        }

        for snake_idx in 0..self.snakes.len() {
            let mut other_bodies = Vec::with_capacity(self.snakes.len() - 1);
            let mut current_snake = None;
            for (i, snake) in self.snakes.iter_mut().enumerate() {
                if i == snake_idx {
                    current_snake = Some(snake)
                } else {
                    other_bodies.push(&snake.body)
                }
            }
            current_snake.unwrap().advance(&other_bodies, &self.apples)
        }

        // for (i, snake) in self.snakes.iter_mut().enumerate() {
        //     let other_snakes = reprs[..i].iter().chain(&reprs[i + 1..]).collect();
        //     snake.advance(other_snakes, &self.apples)
        // }

        // check for crashes
        // [(crashed, into), ...]
        let mut crashed_snake_indices = vec![];
        'outer: for (i, snake) in self.snakes.iter().enumerate() {
            for (j, other) in self.snakes.iter().enumerate() {
                // check head-head crash
                if i != j && snake.body[0].pos == other.body[0].pos {
                    // snake j will be added when it's reached by the outer loop
                    crashed_snake_indices.push((i, j));
                    continue 'outer;
                }

                // check head-body crash (this also checks if a snake crashed with itself)
                for segment in other.body.iter().skip(1) {
                    if snake.body[0].pos == segment.pos {
                        crashed_snake_indices.push((i, j));
                        continue 'outer;
                    }
                }
            }
        }

        if self.prefs.eat_behavior == EatBehavior::Cut
            || self.prefs.eat_behavior == EatBehavior::Mixed
        {
            // TODO: this might do weird things with head-to-head collisions
            for &(i, j) in &crashed_snake_indices {
                if self.prefs.eat_behavior == EatBehavior::Cut || i == j {
                    let crash_point = self.snakes[i].body[0].pos;
                    let drain_start_idx = self.snakes[j]
                        .body
                        .iter()
                        .skip(1)
                        .position(|Hex { pos, .. }| *pos == crash_point)
                        .unwrap_or_else(|| {
                            panic!(
                                "point {:?} not found in snake of index {} (head-to-head collision)",
                                crash_point, j
                            )
                            // TODO: handle this as a special case
                        });

                    let _ = self.snakes[j].body.drain(drain_start_idx + 1..);
                    self.snakes[j].grow = 0;
                }
            }
        }

        if self.prefs.eat_behavior == EatBehavior::Die
            || self.prefs.eat_behavior == EatBehavior::Mixed
        {
            for &(i, j) in &crashed_snake_indices {
                if self.prefs.eat_behavior == EatBehavior::Die || i != j {
                    self.snakes[i].state = SnakeState::Crashed;
                    self.snakes[i].body[0].typ = HexType::Crashed;
                    self.state = GameState::Crashed;
                    self.force_redraw = 10;
                }
            }
        }

        // check apple eating
        let mut k = -1;
        for snake in &mut self.snakes {
            k += 1;
            for i in (0..self.apples.len()).rev() {
                if snake.body.is_empty() {
                    panic!("snake {} is empty", k);
                }
                if snake.body[0].pos == self.apples[i] {
                    self.apples.remove(i);
                    snake.body[0].typ = HexType::Eaten(5);
                }
            }
        }

        self.spawn_apples();

        // thread::yield_now();
        // self.fps.wait_for_control_finish();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // could be fun to implement optionally printing as a square grid
        // with 0, 0 the board is touching top-left (nothing hidden)

        if self.force_redraw > 0 {
            self.force_redraw -= 1;
        } else {
            if !self.fps.maybe_draw() {
                return Ok(());
            }

            if self.state != GameState::Playing {
                return Ok(());
            }
        }

        unsafe {
            static mut T: Option<Instant> = None;
            static mut LAST: Option<VecDeque<f64>> = None;

            if let Some(t) = T {
                if let Some(last) = &mut LAST {
                    let micros = t.elapsed().as_micros();
                    let fps = 1_000_000.0 / micros as f64;
                    if last.len() >= 60 {
                        last.pop_front();
                    }
                    last.push_back(fps);
                    let min = last.iter().copied().fold(f64::NAN, f64::min);
                    let max = last.iter().copied().fold(f64::NAN, f64::max);
                    let avg = last.iter().sum::<f64>() / last.len() as f64;
                    print!(
                        "fps: {:.3} ({:.3} / {:.3}) [{:.3}]      \r",
                        fps, min, max, avg
                    );
                    stdout().flush().unwrap();
                } else {
                    LAST = Some(VecDeque::new());
                }
            }
            T = Some(Instant::now());
        }

        clear(ctx, self.palette.background_color);

        // generate grid mesh first time, later reuse
        if self.grid_mesh.is_none() || self.border_mesh.is_none() {
            let (gm, bm) = self.generate_meshes(ctx)?;
            self.grid_mesh = Some(gm);
            self.border_mesh = Some(bm);
        }
        if self.prefs.draw_grid {
            draw(ctx, self.grid_mesh.as_ref().unwrap(), DrawParam::default())?;
        }
        // draw(ctx, self.border_mesh.as_ref().unwrap(), DrawParam::default())?;

        let builder = &mut MeshBuilder::new();

        // draw snakes, crashed (collision) points on top
        for snake in &mut self.snakes {
            snake.draw_non_crash_points(builder, self.cell_dim)?;
        }

        for snake in &mut self.snakes {
            snake.draw_crash_point(builder, self.cell_dim)?;
        }

        for apple in &self.apples {
            build_cell(builder, *apple, self.palette.apple_color, self.cell_dim)?
        }

        let mesh = &builder.build(ctx)?;
        draw(ctx, mesh, DrawParam::default())?;

        if let Some(Message(ref message, ref mut frames_left)) = self.message {
            let mut text = Text::new(message as &str);
            text.set_font(Font::default(), Scale::uniform(20.));

            let offset = 10.;
            let x = ggez::graphics::drawable_size(ctx).0 - text.width(ctx) as f32 - offset;
            let location = Point2 { x, y: offset };
            let opacity = if *frames_left > 10 {
                1.
            } else {
                *frames_left as f32 / 10.
            };
            let color = Color {
                r: 1.,
                g: 1.,
                b: 1.,
                a: opacity,
            };
            draw(ctx, &text, DrawParam::from((location, color)))?;

            if *frames_left == 0 {
                self.message = None;
            } else {
                *frames_left -= 1;
            }
        }

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
            G => {
                self.prefs.draw_grid = !self.prefs.draw_grid;
                let message = if self.prefs.draw_grid {
                    "Grid on"
                } else {
                    "Grid off"
                };
                self.message = Some(Message(message.to_string(), 100));
            }
            C => {
                let (new_behavior, message) = match self.prefs.eat_behavior {
                    EatBehavior::Cut => (EatBehavior::Mixed, "Mixed"),
                    EatBehavior::Mixed => (EatBehavior::Die, "Die on eat"),
                    EatBehavior::Die => (EatBehavior::Cut, "Cut on eat"),
                };
                self.prefs.eat_behavior = new_behavior;
                self.message = Some(Message(message.to_string(), 100));
            }
            k => {
                if self.state == Playing {
                    for snake in &mut self.snakes {
                        snake.controller.key_pressed(k)
                    }
                }
            }
        }
    }

    // TODO: forbid resizing in-game
    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) {
        let new_dim = Self::wh_to_dim(self.cell_dim, width, height);
        self.dim = new_dim;

        // this is only to allow for hacky mid-game resizing
        for snake in &mut self.snakes {
            snake.board_dim = new_dim;
        }
        // this too
        self.apples.retain(move |apple| apple.is_in(new_dim));
        self.spawn_apples();

        let message = format!("{}x{}", new_dim.h, new_dim.v);
        self.message = Some(Message(message, 100));
        self.grid_mesh = None;

        self.force_redraw = 10; // redraw 10 frames to adjust the grid
    }
}
