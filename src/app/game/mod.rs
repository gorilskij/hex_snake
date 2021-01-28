use ggez::{
    conf::WindowMode,
    event::{EventHandler, KeyCode, KeyMods},
    graphics::{
        clear, draw, present, Color, DrawMode, DrawParam, Font, Mesh, MeshBuilder, Scale, Text,
    },
    Context, GameResult,
};
use hsl::HSL;
use rand::prelude::*;

use crate::{
    app::{
        drawing::{generate_grid_mesh, get_full_hexagon, get_points_animated, SegmentFraction},
        game::game_control::{GameControl, GameState},
        hex::{Dir, HexDim, HexPoint},
        keyboard_control::{ControlSetup, KeyboardLayout, Side},
        palette::GamePalette,
        snake::{
            controller::{OtherSnakes, SnakeController, SnakeControllerTemplate},
            palette::SnakePaletteTemplate,
            EatBehavior, EatMechanics, Segment, SegmentType, Snake, SnakeBody, SnakeSeed,
            SnakeState, SnakeType,
        },
        Frames,
    },
    point::Point,
};

#[derive(Copy, Clone)]
pub struct CellDim {
    pub side: f32,
    // sin is longer than cos
    // they describe the height and width of the diagonal segments of
    // a hexagon with its flat segments horizontal on the top and bottom
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

impl CellDim {
    pub fn center(self) -> Point {
        Point {
            x: self.cos + self.side / 2.,
            y: self.sin,
        }
    }
}

mod game_control {
    use std::{
        collections::VecDeque,
        time::{Duration, Instant},
    };
    use std::cmp::max;

    struct FPSCounter {
        len: usize,
        buffer: VecDeque<Instant>,
    }

    impl FPSCounter {
        fn new(fps: u64) -> Self {
            // TODO: calculate required len in a way that makes sense
            //  maybe also register every n frames for higher fps to avoid wasting space
            let len = max(60, fps as usize);
            Self {
                len,
                buffer: VecDeque::with_capacity(len),
            }
        }

        fn set_fps(&mut self, fps: u64) {
            self.len = max(60, fps as usize);
            self.reset();
        }

        fn register_frame(&mut self) {
            if self.buffer.len() >= self.len {
                self.buffer.pop_front();
            }
            self.buffer.push_back(Instant::now());
        }

        fn reset(&mut self) {
            self.buffer.clear();
        }

        fn fps(&self) -> f64 {
            if self.buffer.len() >= 2 {
                self.buffer.len() as f64
                    / (self.buffer[self.buffer.len() - 1] - self.buffer[0]).as_secs_f64()
            } else {
                0.
            }
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub enum GameState {
        Playing,
        Paused,
        GameOver,
    }

    // combines fps with game state management
    pub struct GameControl {
        game_fps: u64,
        // graphics fps assumed to be 60
        game_frame_duration: Duration,
        graphics_frame_duration: Duration,
        next_game_frame: Option<Instant>,
        next_graphics_frame: Option<Instant>,
        frame_num: usize,

        measured_game_fps: FPSCounter,
        measured_graphics_fps: FPSCounter,

        game_state: GameState,
    }

    impl GameControl {
        pub fn new(fps: u64) -> Self {
            Self {
                game_fps: fps,
                game_frame_duration: Duration::from_micros(1_000_000 / fps),
                graphics_frame_duration: Duration::from_micros(1_000_000 / 60),
                next_game_frame: None,
                next_graphics_frame: None,
                frame_num: 0,

                measured_game_fps: FPSCounter::new(fps),
                measured_graphics_fps: FPSCounter::new(60),

                game_state: GameState::Playing,
            }
        }

        pub fn fps(&self) -> u64 {
            self.game_fps
        }

        pub fn set_fps(&mut self, fps: u64) {
            self.game_fps = fps;
            self.game_frame_duration = Duration::from_micros(1_000_000 / fps);
            self.next_game_frame = Some(Instant::now() + self.game_frame_duration);
            self.measured_game_fps.set_fps(fps);
        }

        // call in update() as while loop condition
        pub fn can_update(&mut self) -> bool {
            let now = Instant::now();
            let next_frame = self.next_game_frame.get_or_insert(now);
            if now >= *next_frame
                && self
                    .next_graphics_frame
                    .map(|ngf| now < ngf)
                    .unwrap_or(true)
            {
                self.measured_game_fps.register_frame();
                *next_frame += self.game_frame_duration;
                self.frame_num += 1;
                true
            } else {
                false
            }
        }

        // call in draw()
        pub fn graphics_frame(&mut self) {
            self.measured_graphics_fps.register_frame();
            self.next_graphics_frame = Some(Instant::now() + self.graphics_frame_duration);
        }

        pub fn state(&self) -> GameState {
            self.game_state
        }

        pub fn play(&mut self) {
            self.game_state = GameState::Playing;
            self.next_game_frame = Some(Instant::now());
            self.measured_game_fps.reset();
        }

        pub fn pause(&mut self) {
            self.game_state = GameState::Paused;
            self.next_game_frame = None;
        }

        pub fn game_over(&mut self) {
            self.game_state = GameState::GameOver;
            self.next_game_frame = None;
        }

        pub fn frame_num(&self) -> usize {
            self.frame_num
        }

        // max(elapsed, frame_duration)
        pub fn elapsed(&self) -> Duration {
            let now = Instant::now();
            match self.next_game_frame {
                Some(inst) if inst >= now => self.game_frame_duration - (inst - now),
                _ => self.game_frame_duration,
            }
        }

        // fraction of the current game frame that has elapsed
        pub fn frame_fraction(&self) -> f32 {
            let f = self.elapsed().as_secs_f32() / self.game_frame_duration.as_secs_f32();
            assert!((0. ..=1.).contains(&f), "f: {}", f);
            f
        }

        pub fn measured_game_fps(&self) -> f64 {
            self.measured_game_fps.fps()
        }

        pub fn measured_graphics_fps(&self) -> f64 {
            self.measured_graphics_fps.fps()
        }
    }
}

type Food = u32;

pub enum AppleType {
    Normal(Food),
    SpawnSnake(SnakeSeed),
}

pub struct Apple {
    pub pos: HexPoint,
    pub typ: AppleType,
}

struct Prefs {
    draw_grid: bool,
    display_fps: bool,
    apple_food: Food,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            draw_grid: true,
            display_fps: false,
            apple_food: 1,
        }
    }
}

struct Message {
    message: String,
    duration: Option<Frames>,
}

pub struct Game {
    control: GameControl,

    window_dim: Point,

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
}

impl Game {
    pub fn new(
        cell_side_len: f32,
        players: Vec<SnakeSeed>,
        palette: GamePalette,
        wm: WindowMode,
    ) -> Self {
        assert!(!players.is_empty(), "No players specified");

        let cell_dim = CellDim::from(cell_side_len);

        let mut game = Self {
            control: GameControl::new(12),

            window_dim: Point {
                x: wm.width,
                y: wm.height,
            },

            dim: HexDim { h: 0, v: 0 }, // placeholder
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
        };
        // warning: this spawns apples before there are any snakes
        game.update_dim();
        game.restart();
        game
    }

    fn update_dim(&mut self) {
        let Point {
            x: width,
            y: height,
        } = self.window_dim;
        let CellDim { side, sin, cos } = self.cell_dim;
        let new_dim = HexDim {
            h: (width / (side + cos)) as isize,
            v: (height / (2. * sin)) as isize - 1,
        };

        if self.dim != new_dim {
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

            // invalidate
            self.grid_mesh = None;
            self.border_mesh = None;
        }
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
                HexPoint {
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
    }

    fn occupied_cells(&self) -> Vec<HexPoint> {
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

    fn random_free_spot(&mut self, occupied_cells: &[HexPoint]) -> Option<HexPoint> {
        let free_spaces = (self.dim.h * self.dim.v) as usize - occupied_cells.len();
        if free_spaces == 0 {
            return None;
        }

        let mut new_idx = self.rng.gen_range(0, free_spaces);
        for HexPoint { h, v } in occupied_cells {
            let idx = (v * self.dim.h + h) as usize;
            if idx <= new_idx {
                new_idx += 1;
            }
        }

        assert!(new_idx < (self.dim.h * self.dim.v) as usize);
        Some(HexPoint {
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
                _ => AppleType::Normal(self.prefs.apple_food),
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
                        self.snakes[snake_idx].die();
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

        // if only ephemeral AIs are left, kill all other snakes
        let dying_or_ephemeral = |snake: &Snake| {
            matches!(snake.state, SnakeState::Dying(_))
                || matches!(
                    snake.snake_type,
                    SnakeType::CompetitorSnake { life: Some(_) }
                        | SnakeType::KillerSnake { life: Some(_) }
                )
        };
        if self.snakes.iter().all(dying_or_ephemeral) {
            for snake in &mut self.snakes {
                snake.die();
            }
        }

        if self.snakes.is_empty() {
            self.control.game_over();
            return;
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
            .filter(|(_, s)| !matches!(s.state, SnakeState::Crashed | SnakeState::Dying(_)))
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
                        self.snakes[i].die();
                        self.snakes[j].die();
                    } else {
                        let drain_start_idx = self.snakes[j]
                            .body
                            .cells
                            .iter()
                            .skip(1)
                            .position(|Segment { pos, .. }| *pos == crash_point)
                            .unwrap_or_else(|| {
                                panic!("point {:?} not found in snake of index {}", crash_point, j)
                            });

                        let _ = self.snakes[j].body.cells.drain(drain_start_idx + 1..);
                        self.snakes[j].body.grow = 0;
                    }
                }
                EatBehavior::Crash => {
                    self.snakes[i].crash();
                    self.control.game_over();
                }
                EatBehavior::Die => {
                    self.snakes[i].die();
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
                        AppleType::Normal(food) => {
                            snake.body.cells[0].typ = SegmentType::Eaten(food)
                        }
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

    fn draw_message(
        maybe_message: &mut Option<Message>,
        side: Side,
        ctx: &mut Context,
    ) -> GameResult {
        if let Some(Message {
            ref message,
            duration: ref mut life,
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
            let location = Point { x, y: offset };

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

impl Game {
    fn draw_snakes_and_apples(&mut self, ctx: &mut Context) -> GameResult {
        let mut builder = MeshBuilder::new();

        // to be drawn later (potentially on top of body segments)
        let mut heads = vec![];

        let frame_frac = if self.control.state() == GameState::Paused {
            // let x = self.fps_limiter.elapsed().as_secs_f32() % 2.;
            // if x > 1. { 2. - x } else { x }
            // TODO: implement some paused animation
            0.5
        } else {
            self.control.frame_fraction()
        };

        for (snake_idx, snake) in self.snakes.iter_mut().enumerate() {
            let len = snake.len();

            if len < 2 {
                println!("warning: snakes of length <= 1 interact weirdly with animation ")
            }

            // draw white aura around snake heads (debug)
            // for pos in snake.head().pos.neighborhood(7) {
            //     let dest = pos.to_point(self.cell_dim);
            //     let color = Color::from_rgb(255, 255, 255);
            //     let translated_points = translate(&hexagon_points, dest);
            //     builder.polygon(DrawMode::fill(), &translated_points, color)?;
            // }

            // TODO: this is weird
            if snake.state == SnakeState::Crashed && snake.head().typ != SegmentType::Crashed {
                println!("idx: {}, len: {}", snake_idx, len);
                panic!("crashed snake with head: {:?}", snake.head().typ)
            }

            let draw_head_separately =
                matches!(snake.state, SnakeState::Crashed | SnakeState::Dying(_));
            let color_offset = match snake.state {
                SnakeState::Dying(offset) => offset,
                _ => 0,
            };

            for (seg_idx, segment) in snake.body.cells.iter().enumerate() {
                let previous = seg_idx
                    .checked_sub(1)
                    .map(|prev_idx| -snake.body.cells[prev_idx].next_segment)
                    .unwrap_or(snake.dir());

                let next = segment.next_segment;

                if draw_head_separately && seg_idx == 0 {
                    heads.push((snake_idx, *segment, previous, next, color_offset));
                    continue;
                }

                // previous = towards head
                // next = towards tail

                let points;
                let dest = segment.pos.to_point(self.cell_dim);
                let color = snake
                    .painter
                    .paint_segment(seg_idx + color_offset, len, segment);

                // TODO: if tail has eaten, animate it more slowly
                let fraction = match seg_idx {
                    0 => SegmentFraction::Appearing(frame_frac),
                    i if i == len - 1
                        && snake.body.grow == 0
                        && !matches!(segment.typ, SegmentType::Eaten(food) if food > 0) =>
                    {
                        SegmentFraction::Disappearing(frame_frac)
                    }
                    _ => SegmentFraction::Solid,
                };

                points = get_points_animated(dest, previous, next, self.cell_dim, fraction);

                builder.polygon(DrawMode::fill(), &points, color)?;
            }
        }

        for (snake_idx, segment, previous, next, color_offset) in heads {
            let Segment { pos, typ, .. } = segment;

            let dest = pos.to_point(self.cell_dim);

            let snake = &mut self.snakes[snake_idx];

            match typ {
                SegmentType::BlackHole => {
                    let hexagon_color = Color::from_rgb(255, 255, 255);
                    let segment_color = snake.painter.paint_segment(
                        color_offset,
                        snake.len() + color_offset,
                        &segment,
                    );
                    let hexagon_points = get_full_hexagon(dest, self.cell_dim);
                    let segment_points = get_points_animated(
                        dest,
                        previous,
                        next,
                        self.cell_dim,
                        SegmentFraction::Appearing(0.5),
                    );
                    builder.polygon(DrawMode::fill(), &hexagon_points, hexagon_color)?;
                    builder.polygon(DrawMode::fill(), &segment_points, segment_color)?;
                }
                SegmentType::Crashed => {
                    let color = snake
                        .painter
                        .paint_segment(0, snake.len(), &snake.body.cells[0]);
                    let points = get_points_animated(
                        dest,
                        previous,
                        next,
                        self.cell_dim,
                        SegmentFraction::Appearing(0.5),
                    );
                    builder.polygon(DrawMode::fill(), &points, color)?;
                }
                _ => unreachable!(
                    "head segment of type {:?} should not have been queued to be drawn separately",
                    typ
                ),
            }
        }

        for apple in &self.apples {
            let dest = apple.pos.to_point(self.cell_dim) + self.cell_dim.center();
            let color = match apple.typ {
                AppleType::Normal(_) => self.palette.apple_color,
                AppleType::SpawnSnake(_) => {
                    let hue = 360. * (self.control.frame_num() % 12) as f64 / 11.;
                    let hsl = HSL {
                        h: hue,
                        s: 1.,
                        l: 0.3,
                    };
                    Color::from(hsl.to_rgb())
                }
            };
            builder.circle(DrawMode::fill(), dest, self.cell_dim.side / 1.5, 0.1, color);
        }

        let mesh = builder.build(ctx)?;
        draw(ctx, &mesh, DrawParam::default())
    }
}

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // it's important that this if go first to reset the last frame time
        // if !self.fps_control.maybe_update() {
        //     // self.fps_control.wait();
        //     return Ok(());
        // }

        // if self.state != GameState::Playing {
        //     self.fps_control.wait();
        //     return Ok(());
        // }

        if matches!(
            self.control.state(),
            GameState::Paused | GameState::GameOver
        ) {
            return Ok(());
        }

        while self.control.can_update() {
            self.advance_snakes();
            self.spawn_apples();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.control.graphics_frame();

        if self.prefs.display_fps {
            self.message_top_left = Some(Message {
                message: format!(
                    "u: {:.2} g: {:.2}",
                    self.control.measured_game_fps(),
                    self.control.measured_graphics_fps(),
                ),
                duration: None,
            })
        }

        clear(ctx, self.palette.background_color);

        if self.prefs.draw_grid {
            if self.grid_mesh.is_none() {
                self.grid_mesh = Some(generate_grid_mesh(
                    ctx,
                    self.dim,
                    &self.palette,
                    self.cell_dim,
                )?);
            };
            draw(ctx, self.grid_mesh.as_ref().unwrap(), DrawParam::default())?;
        }
        // draw(ctx, self.border_mesh.as_ref().unwrap(), DrawParam::default())?;

        self.draw_snakes_and_apples(ctx)?;

        Self::draw_message(&mut self.message_top_left, Side::Left, ctx)?;
        Self::draw_message(&mut self.message_top_right, Side::Right, ctx)?;

        // thread::yield_now();
        present(ctx)
    }

    fn key_down_event(&mut self, _ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
        use KeyCode::*;

        let numeric_keys = [Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9];

        // TODO: also tie these to a keymap
        match key {
            Space => match self.control.state() {
                GameState::GameOver => {
                    self.restart();
                    self.control.play();
                }
                GameState::Playing => self.control.pause(),
                GameState::Paused => self.control.play(),
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
                    duration: Some(100),
                });
            }
            F => {
                self.prefs.display_fps = !self.prefs.display_fps;
                if !self.prefs.display_fps {
                    self.message_top_left = None;
                }
            }
            A => {
                // only apply if there is exactly one player snake
                if self.players.len() == 1 {
                    // hacky
                    unsafe {
                        static mut STASHED_CONTROLLER: Option<Box<dyn SnakeController>> = None;

                        let player_snake = self
                            .snakes
                            .iter_mut()
                            .find(|snake| snake.snake_type == SnakeType::PlayerSnake)
                            .unwrap();

                        let message;
                        match &STASHED_CONTROLLER {
                            None => {
                                STASHED_CONTROLLER = Some(std::mem::replace(
                                    &mut player_snake.controller,
                                    SnakeControllerTemplate::CompetitorAI
                                        .into_controller(player_snake.body.dir),
                                ));
                                message = "Autopilot on";
                            }
                            Some(_) => {
                                player_snake.controller = STASHED_CONTROLLER.take().unwrap();
                                message = "Autopilot off"
                            }
                        }

                        self.message_top_right = Some(Message {
                            message: message.to_string(),
                            duration: Some(100),
                        })
                    }
                }
            }
            LBracket => {
                let new_fps = match self.control.fps() {
                    f if f <= 1 => 1,
                    f if f <= 50 => f - 1,
                    f if f <= 100 => f - 5,
                    f if f <= 250 => f - 10,
                    f if f <= 500 => f - 50,
                    f => f - 100,
                };
                self.control.set_fps(new_fps);
                self.message_top_right = Some(Message {
                    message: format!("fps: {}", new_fps),
                    duration: Some(100),
                })
            }
            RBracket => {
                let new_fps = match self.control.fps() {
                    f if f < 1 => 1,
                    f if f < 50 => f + 1,
                    f if f < 100 => f + 5,
                    f if f < 250 => f + 10,
                    f if f <= 450 => f + 50,
                    f => f + 100,
                };
                self.control.set_fps(new_fps);
                self.message_top_right = Some(Message {
                    message: format!("fps: {}", new_fps),
                    duration: Some(100),
                })
            }
            k if numeric_keys.contains(&k) => {
                let new_food = numeric_keys.iter().position(|nk| *nk == k).unwrap() as Food + 1;
                self.prefs.apple_food = new_food;
                // change existing apples
                for apple in &mut self.apples {
                    if let AppleType::Normal(food) = &mut apple.typ {
                        *food = new_food;
                    }
                }
                self.message_top_right = Some(Message {
                    message: format!("Apple food: {}", new_food),
                    duration: Some(100),
                });
            }
            k @ Down | k @ Up => {
                let factor = if k == Down { 0.9 } else { 1. / 0.9 };
                let mut new_side = self.cell_dim.side * factor;
                if new_side < 1. {
                    new_side = 1.
                }
                if new_side > 20. {
                    new_side = 20.
                }
                self.cell_dim = CellDim::from(new_side);

                self.update_dim();

                self.message_top_right = Some(Message {
                    message: format!("Cell side: {}", new_side),
                    duration: Some(100),
                });
            }
            k => {
                if self.control.state() == GameState::Playing {
                    for snake in &mut self.snakes {
                        snake.controller.key_pressed(k)
                    }
                }
            }
        }
    }

    // TODO: forbid resizing in-game
    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) {
        self.window_dim = Point {
            x: width,
            y: height,
        };
        self.update_dim();
        let HexDim { h, v } = self.dim;
        self.message_top_right = Some(Message {
            message: format!("{}x{}", h, v),
            duration: Some(100),
        });
    }
}
