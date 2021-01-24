use std::{
    cmp::min,
    collections::VecDeque,
    mem, thread,
    time::{Duration, Instant},
};

use ggez::{
    conf::WindowMode,
    event::{EventHandler, KeyCode, KeyMods},
    graphics::{
        clear, draw, present, Color, DrawMode, DrawParam, Font, Mesh, MeshBuilder, Scale, Text,
    },
    mint::Point2,
    Context, GameResult,
};
use hsl::HSL;
use num_integer::Integer;
use rand::prelude::*;

use crate::{
    app::{
        hex::{Dir, Hex, HexDim, HexPos, HexType},
        keyboard_control::Side,
        palette::GamePalette,
        snake::{
            controller::{OtherSnakes, SnakeControllerTemplate},
            palette::SnakePaletteTemplate,
            EatBehavior, EatMechanics, Snake, SnakeSeed, SnakeState, SnakeType,
        },
        Frames,
    },
    // times::Times,
};
// use ggez::graphics::{spritebatch::SpriteBatch, Image};
// use itertools::Itertools;

// TODO document
#[derive(Copy, Clone)]
pub struct CellDim {
    pub side: f32,
    pub sin: f32,
    pub cos: f32,
}

impl From<f32> for CellDim {
    fn from(side: f32) -> Self {
        use std::f32::consts::FRAC_PI_3;
        Self {
            side,
            sin: FRAC_PI_3.sin() * side,
            cos: FRAC_PI_3.cos() * side,
        }
    }
}

struct FPSCounter {
    buffer: VecDeque<f64>,
    last: Instant,
}

impl FPSCounter {
    // number of last frames to average
    const N: usize = 60;

    fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(Self::N),
            last: Instant::now(),
        }
    }

    fn register_frame(&mut self) {
        let last = mem::replace(&mut self.last, Instant::now());
        if self.buffer.len() >= Self::N {
            self.buffer.pop_front();
        }
        let fps = 1_000_000. / last.elapsed().as_micros() as f64;
        self.buffer.push_back(fps);
    }

    fn fps(&self) -> f64 {
        self.buffer.iter().sum::<f64>() / self.buffer.len() as f64
    }
}

// used to tie update frames to drawing frames and maintain a reduced framerate
struct FPSControl {
    ggez_frames: Frames, // incremented every time ggez calls update()
    game_frames: Frames, // incremented every time a frame is actually calculated/drawn

    frame_duration: Duration,   // for update and draw
    control_duration: Duration, // for key events (more frequent)
    last_frame: Option<Instant>,
    drawn: bool,
}

impl FPSControl {
    // TODO: unify naming
    //  ggez_frames relates to control_fps and control_duration
    //  game_frames relates to update_fps and frame_duration
    fn new(update_fps: u64, control_fps: u64) -> Self {
        Self {
            ggez_frames: 0,
            game_frames: 0,

            frame_duration: Duration::from_micros(1_000_000 / update_fps),
            control_duration: Duration::from_micros(1_000_000 / control_fps),
            // last_frame: Instant::now(),
            last_frame: None,
            drawn: false,
        }
    }

    // expected to be called every time the game is updated
    // for this game logic and graphics frames are the same
    fn maybe_update(&mut self) -> bool {
        // NOTE: this keeps being called when the game is paused
        // maybe implement a more relaxed control fps for pause

        self.ggez_frames += 1;

        let last_frame = self.last_frame.get_or_insert(Instant::now());
        let can_update = last_frame.elapsed() >= self.frame_duration;
        if can_update {
            self.game_frames += 1;

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

#[derive(Eq, PartialEq)]
enum GameState {
    Playing,
    Paused,
    Crashed,
}

struct Prefs {
    draw_grid: bool,
    display_fps: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            draw_grid: true,
            display_fps: false,
        }
    }
}

struct Message {
    message: String,
    life: Option<Frames>,
}

pub enum AppleType {
    Normal(u32),
    SpawnSnake(SnakeSeed),
}

pub struct Apple {
    pub pos: HexPos,
    pub typ: AppleType,
}

pub struct Game {
    state: GameState,
    fps_control: FPSControl,
    graphics_fps: FPSCounter,

    dim: HexDim,
    players: Vec<SnakeSeed>,
    snakes: Vec<Snake>,
    apples: Vec<Apple>,

    cell_dim: CellDim,
    palette: GamePalette,

    apple_count: usize,

    rng: ThreadRng,
    grid_mesh: Option<Mesh>,
    border_mesh: Option<Mesh>,

    prefs: Prefs,
    message_top_left: Option<Message>,
    message_top_right: Option<Message>,

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
            fps_control: FPSControl::new(12, 60),
            // fps_control: FPSControl::new(240, 240),
            graphics_fps: FPSCounter::new(),

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
            message_top_left: None,
            message_top_right: None,

            force_redraw: 0,
        };
        game.restart();
        game
    }

    fn restart(&mut self) {
        self.snakes.clear();
        self.apples.clear();

        // const DISTANCE_BETWEEN_SNAKES: isize = 10;
        const DISTANCE_BETWEEN_SNAKES: isize = 1;

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
            ));

            // alternate
            dir = -dir;
        }

        self.spawn_apples();
        self.state = GameState::Playing;
    }

    fn occupied_cells(&self) -> Vec<HexPos> {
        // upper bound
        let max_occupied_cells =
            self.snakes.iter().map(|snake| snake.len()).sum::<usize>() + self.apples.len();
        let mut occupied_cells = Vec::with_capacity(max_occupied_cells);
        occupied_cells.extend(self.apples.iter().map(|Apple { pos, .. }| pos));
        for snake in &self.snakes {
            occupied_cells.extend(snake.body.cells.iter().map(|hex| hex.pos));
        }
        occupied_cells.sort_unstable();
        occupied_cells.dedup();
        occupied_cells
    }

    fn random_free_spot(&mut self, occupied_cells: &[HexPos]) -> Option<HexPos> {
        let free_spaces = (self.dim.h * self.dim.v) as usize - occupied_cells.len();
        if free_spaces == 0 {
            return None;
        }

        let mut new_idx = self.rng.gen_range(0, free_spaces);
        for HexPos { h, v } in occupied_cells {
            let idx = (v * self.dim.h + h) as usize;
            if idx <= new_idx {
                new_idx += 1;
            }
        }

        assert!(new_idx < (self.dim.h * self.dim.v) as usize);
        Some(HexPos {
            h: new_idx as isize % self.dim.h,
            v: new_idx as isize / self.dim.h,
        })
    }

    pub fn spawn_apples(&mut self) {
        let mut occupied_cells = self.occupied_cells();

        while self.apples.len() < self.apple_count {
            let apple_pos = match self.random_free_spot(&occupied_cells) {
                Some(pos) => pos,
                None => {
                    println!(
                        "warning: no space left for new apples ({} apples will be missing)",
                        self.apple_count - self.apples.len()
                    );
                    return;
                }
            };

            // insert at sorted position
            match occupied_cells.binary_search(&apple_pos) {
                Ok(idx) => panic!("Spawned apple at occupied cell {:?}", occupied_cells[idx]),
                Err(idx) => occupied_cells.insert(idx, apple_pos),
            }

            let apple_type = match self.rng.gen::<f32>() {
                x if x < 0.025 => AppleType::SpawnSnake(SnakeSeed {
                    snake_type: SnakeType::CompetitorSnake { life: Some(200) },
                    eat_mechanics: EatMechanics::always(EatBehavior::Die),
                    palette: SnakePaletteTemplate::new_persistent_pastel_rainbow(),
                    controller: SnakeControllerTemplate::CompetitorAI,
                }),
                x if x < 0.040 => {
                    if !self
                        .snakes
                        .iter()
                        .any(|s| s.snake_type == SnakeType::PlayerSnake)
                    {
                        println!("warning: didn't spawn killer snake apple because there is no player snake");
                        AppleType::Normal(1)
                    } else {
                        AppleType::SpawnSnake(SnakeSeed {
                            snake_type: SnakeType::KillerSnake { life: Some(200) },
                            eat_mechanics: EatMechanics::always(EatBehavior::Die),
                            palette: SnakePaletteTemplate::new_persistent_dark_rainbow(),
                            controller: SnakeControllerTemplate::KillerAI,
                        })
                    }
                }
                _ => AppleType::Normal(1),
            };

            self.apples.push(Apple {
                pos: apple_pos,
                typ: apple_type,
            });
        }
    }

    fn advance_snakes(&mut self) {
        let mut remove_snakes = vec![];
        for snake_idx in 0..self.snakes.len() {
            // set snake to die if it ran out of life
            match &mut self.snakes[snake_idx].snake_type {
                SnakeType::CompetitorSnake { life: Some(life) }
                | SnakeType::KillerSnake { life: Some(life) } => {
                    if *life == 0 {
                        self.snakes[snake_idx].state = SnakeState::Dying;
                        self.snakes[snake_idx].body.cells[0].typ = HexType::BlackHole;
                    } else {
                        *life -= 1;
                    }
                }
                _ => (),
            }

            // advance the snake
            let (other_snakes1, rest) = self.snakes.split_at_mut(snake_idx);
            let (snake, other_snakes2) = rest.split_at_mut(1);
            let snake = &mut snake[0];
            snake.advance(
                OtherSnakes(other_snakes1, other_snakes2),
                &self.apples,
                self.dim,
            );

            // remove snake if it ran out of body
            if snake.len() == 0 {
                remove_snakes.push(snake_idx);
            }
        }

        remove_snakes.sort_unstable();
        for snake_idx in remove_snakes.into_iter().rev() {
            self.snakes.remove(snake_idx);
        }

        // check for crashes
        // [(index of snake that crashed, index of snake into which it crashed), ...]
        let mut crashed_snake_indices = vec![];
        // checks if snake i crashed into snake j
        // crashed and dying snakes can be ignored for i
        'outer: for (i, snake) in self
            .snakes
            .iter()
            .enumerate()
            .filter(|(_, s)| s.state != SnakeState::Crashed && s.state != SnakeState::Dying)
        {
            for (j, other) in self.snakes.iter().enumerate() {
                // check head-head crash
                if i != j && snake.head().pos == other.head().pos {
                    // snake j will be added when it's reached by the outer loop
                    crashed_snake_indices.push((i, j));
                    continue 'outer;
                }

                // check head-body crash (this also checks if a snake crashed with itself)
                for segment in other.body.cells.iter().skip(1) {
                    if snake.head().pos == segment.pos {
                        crashed_snake_indices.push((i, j));
                        continue 'outer;
                    }
                }
            }
        }

        for &(i, j) in &crashed_snake_indices {
            let mechanics = &self.snakes[i].eat_mechanics;
            let behavior;
            if i == j {
                behavior = mechanics.eat_self;
            } else {
                let other_snake_type = &self.snakes[j].snake_type;
                behavior = mechanics
                    .eat_other
                    .get(other_snake_type)
                    .copied()
                    .unwrap_or(mechanics.default);
            }

            match behavior {
                EatBehavior::Cut => {
                    let crash_point = self.snakes[i].head().pos;
                    if i != j && crash_point == self.snakes[j].head().pos {
                        // special case for a head-to-head collision, can't cut..
                        println!("warning: invoked head-to-head collision special case");
                        self.snakes[i].state = SnakeState::Dying;
                        self.snakes[i].body.cells[0].typ = HexType::BlackHole;
                        self.snakes[j].state = SnakeState::Dying;
                        self.snakes[j].body.cells[0].typ = HexType::BlackHole;
                    } else {
                        let drain_start_idx = self.snakes[j]
                            .body
                            .cells
                            .iter()
                            .skip(1)
                            .position(|Hex { pos, .. }| *pos == crash_point)
                            .unwrap_or_else(|| {
                                panic!("point {:?} not found in snake of index {}", crash_point, j)
                            });

                        let _ = self.snakes[j].body.cells.drain(drain_start_idx + 1..);
                        self.snakes[j].body.grow = 0;
                    }
                }
                EatBehavior::Crash => {
                    self.snakes[i].state = SnakeState::Crashed;
                    self.snakes[i].body.cells[0].typ = HexType::Crashed;
                    self.state = GameState::Crashed;
                    self.force_redraw = 10;
                }
                EatBehavior::Die => {
                    self.snakes[i].state = SnakeState::Dying;
                    self.snakes[i].body.cells[0].typ = HexType::BlackHole;
                }
            }
        }

        // check apple eating
        let mut k = -1;
        let mut spawn_snake = None;
        for snake in &mut self.snakes {
            k += 1;
            for i in (0..self.apples.len()).rev() {
                if snake.len() == 0 {
                    panic!("snake {} is empty", k);
                }
                if snake.head().pos == self.apples[i].pos {
                    let Apple { typ, .. } = self.apples.remove(i);
                    match typ {
                        AppleType::Normal(food) => snake.body.cells[0].typ = HexType::Eaten(food),
                        AppleType::SpawnSnake(seed) => spawn_snake = Some(seed),
                    }
                }
            }
        }

        if let Some(seed) = spawn_snake {
            // avoid spawning too close to player snake heads
            const PLAYER_SNAKE_HEAD_NO_SPAWN_RADIUS: usize = 7;

            let mut occupied_cells = self.occupied_cells();
            for snake in self
                .snakes
                .iter()
                .filter(|s| s.snake_type == SnakeType::PlayerSnake)
            {
                let neighborhood = snake
                    .head()
                    .pos
                    .neighborhood(PLAYER_SNAKE_HEAD_NO_SPAWN_RADIUS);
                occupied_cells.extend_from_slice(&neighborhood);
            }
            occupied_cells.sort_unstable();
            occupied_cells.dedup();

            if let Some(pos) = self.random_free_spot(&occupied_cells) {
                self.snakes
                    .push(Snake::from_seed(&seed, pos, Dir::random(&mut self.rng), 10))
            } else {
                println!("warning: failed to spawn evil snake, no free spaces left")
            }
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

    fn draw_message(
        maybe_message: &mut Option<Message>,
        side: Side,
        ctx: &mut Context,
    ) -> GameResult {
        if let Some(Message {
            ref message,
            ref mut life,
        }) = maybe_message
        {
            let mut text = Text::new(message as &str);
            text.set_font(Font::default(), Scale::uniform(20.));

            let offset = 10.;
            let x = match side {
                Side::Left => 10.,
                Side::Right => {
                    ggez::graphics::drawable_size(ctx).0 - text.width(ctx) as f32 - offset
                }
            };
            let location = Point2 { x, y: offset };

            let mut opacity = 1.;
            if let Some(frames) = life {
                if *frames < 10 {
                    opacity = *frames as f32 / 10.
                }
            }

            let color = Color {
                r: 1.,
                g: 1.,
                b: 1.,
                a: opacity,
            };
            draw(ctx, &text, DrawParam::from((location, color)))?;

            if let Some(frames) = life {
                if *frames == 0 {
                    *maybe_message = None;
                } else {
                    *frames -= 1;
                }
            }
        }
        Ok(())
    }
}

// TODO: refactor these out of Game
#[rustfmt::skip]
fn get_points(dest: Point2<f32>, from: Option<Dir>, to: Option<Dir>, cell_dim: CellDim) -> Vec<Point2<f32>> {
    let CellDim { side, sin, cos } = cell_dim;

    let mut points = if from == Some(Dir::D) && to == Some(Dir::U) {
        vec![
            Point2 { x: cos, y: 0. },
            Point2 { x: side + cos, y: 0., },
            Point2 { x: side + cos, y: 2. * sin, },
            Point2 { x: cos, y: 2. * sin, },
        ]
    } else {
        vec![
            Point2 { x: cos, y: 0. },
            Point2 { x: side + cos, y: 0., },
            Point2 { x: side + 2. * cos, y: sin, },
            Point2 { x: side + cos, y: 2. * sin, },
            Point2 { x: cos, y: 2. * sin, },
            Point2 { x: 0., y: sin },
        ]
    };

    for Point2 { x, y } in &mut points {
        *x += dest.x;
        *y += dest.y;
    }

    points
}

type HexagonPoints = [Point2<f32>; 6];
impl Game {

    pub fn get_hexagon_points(&self) -> HexagonPoints {
        let CellDim { side, sin, cos } = self.cell_dim;

        unsafe {
            static mut CACHED_HEXAGON_POINTS: Option<(f32, HexagonPoints)> = None;

            if let Some((cached_side, points)) = CACHED_HEXAGON_POINTS {
                if (side - cached_side).abs() < f32::EPSILON {
                    return points;
                }
            }

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

    #[allow(dead_code)]
    fn draw_snakes_and_apples_tesselation(&mut self, ctx: &mut Context) -> GameResult {
        let mut builder = MeshBuilder::new();
        // let hexagon_points = self.get_hexagon_points();

        // fn translate(points: &[Point2<f32>], dest: Point2<f32>) -> Vec<Point2<f32>> {
        //     points
        //         .iter()
        //         .map(|Point2 { x, y }| Point2 {
        //             x: x + dest.x,
        //             y: y + dest.y,
        //         })
        //         .collect()
        // }

        for snake in &mut self.snakes {
            let len = snake.len();

            // draw white aura around snake heads (debug)
            // for pos in snake.head().pos.neighborhood(7) {
            //     let dest = pos.to_point(self.cell_dim);
            //     let color = Color::from_rgb(255, 255, 255);
            //     let translated_points = translate(&hexagon_points, dest);
            //     builder.polygon(DrawMode::fill(), &translated_points, color)?;
            // }

            for (seg_idx, hex) in snake.body.cells.iter().enumerate() {
                // closer to head
                let next = if seg_idx == 0 {
                    None
                } else {
                    Some(snake.body.cells[seg_idx - 1].pos)
                };

                // closer to tail
                let previous = snake.body.cells.get(seg_idx + 1).map(|h| h.pos);

                let from = previous.and_then(|prev| hex.pos.exact_dir_to(prev, 1));
                let to = next.and_then(|nxt| hex.pos.exact_dir_to(nxt, 1));

                println!("{:?} / {:?}", from, to);

                let dest = hex.pos.to_point(self.cell_dim);
                let color = snake.painter.paint_segment(seg_idx, len, hex);
                // let translated_points = translate(&hexagon_points, dest);
                let points = get_points(dest, from, to, self.cell_dim);
                builder.polygon(DrawMode::fill(), &points, color)?;
            }
        }

        // for snake in self
        //     .snakes
        //     .iter_mut()
        //     .filter(|snake| snake.state == SnakeState::Crashed || snake.state == SnakeState::Dying)
        // {
        //     let dest = snake.head().pos.to_point(self.cell_dim);
        //     let color = snake
        //         .painter
        //         .paint_segment(0, snake.len(), &snake.body.cells[0]);
        //     // let translated_points = translate(&hexagon_points, dest);
        //     builder.polygon(DrawMode::fill(), &translated_points, color)?;
        // }

        // for apple in &self.apples {
        //     let dest = apple.pos.to_point(self.cell_dim);
        //     let color = match apple.typ {
        //         AppleType::Normal(_) => self.palette.apple_color,
        //         AppleType::SpawnSnake(_) => {
        //             let hue = 360. * (self.fps_control.game_frames % 12) as f64 / 11.;
        //             let hsl = HSL {
        //                 h: hue,
        //                 s: 1.,
        //                 l: 0.3,
        //             };
        //             Color::from(hsl.to_rgb())
        //         }
        //     };
        //     let translated_points = translate(&hexagon_points, dest);
        //     builder.polygon(DrawMode::fill(), &translated_points, color)?;
        // }

        let mesh = builder.build(ctx)?;
        draw(ctx, &mesh, DrawParam::default())
    }

    // outdated
    // fn get_hexagon_image(&self, ctx: &mut Context) -> Image {
    //     let CellDim { side, sin, cos } = self.cell_dim;
    //
    //     unsafe {
    //         static mut HEXAGON: Option<(f32, Image)> = None;
    //
    //         if let Some((cached_side, image)) = &HEXAGON {
    //             if (*cached_side - side).abs() < f32::EPSILON {
    //                 return image.clone();
    //             }
    //         }
    //
    //         let width = (2. * cos + side) as u16;
    //         let height = (2. * sin) as u16;
    //         let overflow = 0.02;
    //         let hexagon = (0..height)
    //             .cartesian_product(0..width)
    //             .map(|(y, x)| {
    //                 // check if the point is in bounds of the four relevant hexagon edges
    //                 // top and bottom edges can't be violated because the size of the image is already restricted
    //                 let x = x as f32;
    //                 let y = y as f32;
    //                 let top_left = y <= (sin / cos) * x + sin + overflow;
    //                 let top_right = y <= -sin / cos * (x - 2. * cos - side) + sin + overflow;
    //                 let bottom_left = y >= -sin / cos * x + sin - overflow;
    //                 let bottom_right = y >= sin / cos * (x - 2. * cos - side) + sin - overflow;
    //                 if top_left && top_right && bottom_left && bottom_right {
    //                     // if y < x {
    //                     255
    //                 } else {
    //                     0
    //                 }
    //             })
    //             .times(4)
    //             .collect::<Vec<_>>();
    //         let image = Image::from_rgba8(ctx, width, height, &hexagon).unwrap();
    //         HEXAGON = Some((side, image.clone()));
    //         image
    //     }
    // }

    // #[allow(dead_code)]
    // fn draw_snakes_and_apples_sprite_batch(&mut self, ctx: &mut Context) -> GameResult {
    //     let image = self.get_hexagon_image(ctx);
    //     let mut sprite_batch = SpriteBatch::new(image);
    //
    //     // draw snakes with collision points on top
    //     for snake in &mut self.snakes {
    //         let len = snake.len();
    //         for (seg_idx, hex) in snake.body.cells.iter().enumerate() {
    //             let dest = hex.pos.to_point(self.cell_dim);
    //             let color = snake.painter.paint_segment(seg_idx, len, hex);
    //             sprite_batch.add(DrawParam::new().dest(dest).color(color));
    //         }
    //     }
    //
    //     for snake in self
    //         .snakes
    //         .iter_mut()
    //         .filter(|snake| snake.state == SnakeState::Crashed || snake.state == SnakeState::Dying)
    //     {
    //         let dest = snake.head().pos.to_point(self.cell_dim);
    //         let color = snake
    //             .painter
    //             .paint_segment(0, snake.len(), &snake.body.cells[0]);
    //         sprite_batch.add(DrawParam::new().dest(dest).color(color));
    //     }
    //
    //     for apple in &self.apples {
    //         let dest = apple.pos.to_point(self.cell_dim);
    //         let color = match apple.typ {
    //             AppleType::Normal(_) => self.palette.apple_color,
    //             AppleType::SpawnSnake(_) => {
    //                 let hue = 360. * (self.fps_control.game_frames % 12) as f64 / 11.;
    //                 let hsl = HSL {
    //                     h: hue,
    //                     s: 1.,
    //                     l: 0.3,
    //                 };
    //                 Color::from(hsl.to_rgb())
    //             }
    //         };
    //         sprite_batch.add(DrawParam::new().dest(dest).color(color));
    //     }
    //
    //     draw(ctx, &sprite_batch, DrawParam::default())
    // }
}

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // it's important that this if go first to reset the last frame time
        if !self.fps_control.maybe_update() {
            self.fps_control.wait();
            return Ok(());
        }

        if self.state != GameState::Playing {
            self.fps_control.wait();
            return Ok(());
        }

        self.advance_snakes();
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
            if !self.fps_control.maybe_draw() {
                return Ok(());
            }

            if self.state != GameState::Playing {
                return Ok(());
            }
        }

        // objective counting of when graphics frames actually occur
        self.graphics_fps.register_frame();
        if self.prefs.display_fps {
            self.message_top_left = Some(Message {
                message: format!("{:.2}", self.graphics_fps.fps()),
                life: None,
            })
        }

        // skip frames for faster gameplay
        // unsafe {
        //     static mut MG: u64 = 0;
        //     MG += 1;
        //     if MG % 5 != 0 { return Ok(()) }
        // }

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

        // self.draw_snakes_and_apples_sprite_batch(ctx)?;
        self.draw_snakes_and_apples_tesselation(ctx)?;

        Self::draw_message(&mut self.message_top_left, Side::Left, ctx)?;
        Self::draw_message(&mut self.message_top_right, Side::Right, ctx)?;

        thread::yield_now();
        present(ctx)
    }

    fn key_down_event(&mut self, _ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
        use GameState::*;
        use KeyCode::*;

        // TODO: also tie these to a keymap
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
                self.message_top_right = Some(Message {
                    message: message.to_string(),
                    life: Some(100),
                });
            }
            F => {
                self.prefs.display_fps = !self.prefs.display_fps;
                if !self.prefs.display_fps {
                    self.message_top_left = None;
                }
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

        // restart if player snake head has left board limits
        if self
            .snakes
            .iter()
            .any(|s| s.snake_type == SnakeType::PlayerSnake && !new_dim.contains(s.head().pos))
        {
            println!("warning: player snake outside of board, restarting");
            self.restart();
        } else {
            // remove snakes outside of board limits
            self.snakes
                .retain(move |snake| new_dim.contains(snake.head().pos));

            // remove apples outside of board limits
            self.apples.retain(move |apple| new_dim.contains(apple.pos));
            self.spawn_apples();
        }

        let message = format!("{}x{}", new_dim.h, new_dim.v);
        self.message_top_right = Some(Message {
            message,
            life: Some(100),
        });
        self.grid_mesh = None;

        self.force_redraw = 10; // redraw 10 frames to adjust the grid
    }
}
