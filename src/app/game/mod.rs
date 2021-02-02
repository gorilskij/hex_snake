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
        apple_spawn_strategy::{AppleSpawn, AppleSpawnStrategy},
        drawing::{generate_grid_mesh, get_full_hexagon, get_points_animated, SegmentFraction},
        game::game_control::{GameControl, GameState},
        palette::GamePalette,
        snake::{
            controller::{OtherSnakes, SnakeController, SnakeControllerTemplate},
            palette::SnakePaletteTemplate,
            EatBehavior, EatMechanics, Segment, SegmentType, Snake, SnakeSeed, SnakeState,
            SnakeType,
        },
        Frames,
    },
    basic::{CellDim, Dir, HexDim, HexPoint, Point, Side},
};
use ggez::graphics::WHITE;

mod game_control;

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
    message_duration: Frames,
    // full hexagons, no animation
    old_style_graphics: bool,
    special_apples: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            draw_grid: false,
            display_fps: false,
            apple_food: 1,
            message_duration: 100,
            old_style_graphics: false,
            special_apples: true,
        }
    }
}

struct Message {
    message: String,
    duration: Option<Frames>,
    color: Color,
}

impl From<(String, Frames)> for Message {
    fn from((message, life): (String, Frames)) -> Self {
        Self {
            message,
            duration: Some(life),
            color: WHITE,
        }
    }
}

impl From<(String, Color)> for Message {
    fn from((message, color): (String, Color)) -> Self {
        Self {
            message,
            duration: None,
            color,
        }
    }
}

pub struct Game {
    control: GameControl,

    window_dim: Point,

    dim: HexDim,
    seeds: Vec<SnakeSeed>,
    snakes: Vec<Snake>,
    apples: Vec<Apple>,

    apple_spawn_strategy: AppleSpawnStrategy,

    cell_dim: CellDim,
    palette: GamePalette,

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
        seeds: Vec<SnakeSeed>,
        palette: GamePalette,
        apple_spawn_strategy: AppleSpawnStrategy,
        wm: WindowMode,
    ) -> Self {
        assert!(!seeds.is_empty(), "No players specified");

        let cell_dim = CellDim::from(cell_side_len);

        let mut game = Self {
            control: GameControl::new(12),

            window_dim: Point {
                x: wm.width,
                y: wm.height,
            },

            dim: HexDim { h: 0, v: 0 }, // placeholder
            seeds: seeds.into_iter().map(Into::into).collect(),
            snakes: vec![],
            apples: vec![],

            apple_spawn_strategy,

            cell_dim,
            palette,

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

        // seeds without a defined spawn point
        let unpositioned = self
            .seeds
            .iter()
            .filter(|seed| !matches!(seed.snake_type, SnakeType::SimulatedSnake { .. }))
            .count();

        let mut unpositioned_dir = Dir::U;
        let mut unpositioned_h_pos: Box<dyn Iterator<Item = isize>> = if unpositioned > 0 {
            const DISTANCE_BETWEEN_SNAKES: isize = 1;

            let total_width = (unpositioned - 1) as isize * DISTANCE_BETWEEN_SNAKES + 1;
            assert!(total_width < self.dim.h, "snakes spread too wide");

            let half = total_width / 2;
            let middle = self.dim.h / 2;
            let start = middle - half;
            let end = start + total_width - 1;

            Box::new((start..=end).step_by(DISTANCE_BETWEEN_SNAKES as usize))
        } else {
            Box::new(std::iter::empty())
        };

        for seed in self.seeds.iter() {
            match seed.snake_type {
                SnakeType::SimulatedSnake {
                    start_pos,
                    start_dir,
                    start_grow,
                } => {
                    self.snakes
                        .push(Snake::from_seed(seed, start_pos, start_dir, start_grow));
                }
                _ => {
                    self.snakes.push(Snake::from_seed(
                        seed,
                        HexPoint {
                            h: unpositioned_h_pos.next().unwrap(),
                            v: self.dim.v / 2,
                        },
                        unpositioned_dir,
                        10,
                    ));

                    // alternate
                    unpositioned_dir = -unpositioned_dir;
                }
            }
        }

        let left = unpositioned_h_pos.count();
        assert_eq!(left, 0, "unexpected iterator length");

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

    fn random_free_spot(
        occupied_cells: &[HexPoint],
        board_dim: HexDim,
        rng: &mut impl Rng,
    ) -> Option<HexPoint> {
        let free_spaces = (board_dim.h * board_dim.v) as usize - occupied_cells.len();
        if free_spaces == 0 {
            return None;
        }

        let mut new_idx = rng.gen_range(0, free_spaces);
        for HexPoint { h, v } in occupied_cells {
            let idx = (v * board_dim.h + h) as usize;
            if idx <= new_idx {
                new_idx += 1;
            }
        }

        assert!(new_idx < (board_dim.h * board_dim.v) as usize);
        Some(HexPoint {
            h: new_idx as isize % board_dim.h,
            v: new_idx as isize / board_dim.h,
        })
    }

    fn generate_apple(&mut self, apple_pos: HexPoint) {
        let apple_type = if self.prefs.special_apples {
            match self.rng.gen::<f32>() {
                x if x < 0.025 => {
                    let controller = if self.rng.gen::<f32>() < 0.5 {
                        SnakeControllerTemplate::CompetitorAI
                    } else {
                        SnakeControllerTemplate::CompetitorAI2
                    };
                    AppleType::SpawnSnake(SnakeSeed {
                        snake_type: SnakeType::CompetitorSnake { life: Some(200) },
                        eat_mechanics: EatMechanics::always(EatBehavior::Die),
                        palette: SnakePaletteTemplate::pastel_rainbow().persistent(),
                        controller,
                    })
                }
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
                            palette: SnakePaletteTemplate::dark_blue_to_red(),
                            controller: SnakeControllerTemplate::KillerAI,
                        })
                    }
                }
                _ => AppleType::Normal(self.prefs.apple_food),
            }
        } else {
            AppleType::Normal(self.prefs.apple_food)
        };

        self.apples.push(Apple {
            pos: apple_pos,
            typ: apple_type,
        });
    }

    pub fn spawn_apples(&mut self) {
        // lazy
        let mut occupied_cells = None;

        loop {
            let can_spawn = match self.apple_spawn_strategy {
                AppleSpawnStrategy::Random { apple_count } => self.apples.len() < apple_count,
                AppleSpawnStrategy::ScheduledOnEat { apple_count, .. } => {
                    self.apples.len() < apple_count
                }
            };

            if !can_spawn {
                break;
            }

            let occupied_cells = occupied_cells.get_or_insert_with(|| self.occupied_cells());

            let apple_pos = match &mut self.apple_spawn_strategy {
                AppleSpawnStrategy::Random { apple_count } => {
                    let apple_pos =
                        match Self::random_free_spot(&occupied_cells, self.dim, &mut self.rng) {
                            Some(pos) => pos,
                            None => {
                                println!(
                                "warning: no space left for new apples ({} apples will be missing)",
                                *apple_count - self.apples.len()
                            );
                                return;
                            }
                        };

                    // insert at sorted position
                    match occupied_cells.binary_search(&apple_pos) {
                        Ok(idx) => {
                            panic!("Spawned apple at occupied cell {:?}", occupied_cells[idx])
                        }
                        Err(idx) => occupied_cells.insert(idx, apple_pos),
                    }

                    Some(apple_pos)
                }
                AppleSpawnStrategy::ScheduledOnEat {
                    spawns, next_index, ..
                } => {
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
            };

            match apple_pos {
                Some(pos) => self.generate_apple(pos),
                None => break,
            }
        }
    }

    // mut snake at idk and immutable all other snakes
    fn split_snakes(snakes: &mut [Snake], idx: usize) -> (&mut Snake, OtherSnakes) {
        let (other_snakes1, rest) = snakes.split_at_mut(idx);
        let (snake, other_snakes2) = rest.split_at_mut(1);
        let snake = &mut snake[0];
        (snake, OtherSnakes(other_snakes1, other_snakes2))
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

            let (snake, other_snakes) = Self::split_snakes(&mut self.snakes, snake_idx);

            // advance the snake
            snake.advance(other_snakes, &self.apples, self.dim);

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
                        // TODO: cause only a crash if even one of the snakes crashed
                        // TODO: what happens if a snake encounters a black hole in progress?
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

                        // ensure a length of at least 2 to avoid weird animation
                        if self.snakes[j].len() < 2 {
                            self.snakes[j].body.grow = 2 - self.snakes.len();
                        } else {
                            self.snakes[j].body.grow = 0;
                        }
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
        for snake in self
            .snakes
            .iter_mut()
            .filter(|snake| snake.state == SnakeState::Living)
        {
            k += 1;
            for i in (0..self.apples.len()).rev() {
                if snake.len() == 0 {
                    panic!("snake {} is empty", k);
                }
                if snake.head().pos == self.apples[i].pos {
                    let Apple { typ, .. } = self.apples.remove(i);
                    match typ {
                        AppleType::Normal(food) => {
                            snake.body.cells[0].typ = SegmentType::Eaten {
                                original_food: food,
                                food_left: food,
                            }
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
                let neighborhood =
                    snake.reachable(PLAYER_SNAKE_HEAD_NO_SPAWN_RADIUS, self.dim);
                occupied_cells.extend_from_slice(&neighborhood);
            }
            occupied_cells.sort_unstable();
            occupied_cells.dedup();

            if let Some(pos) = Self::random_free_spot(&occupied_cells, self.dim, &mut self.rng) {
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
            color,
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

            let mut color = *color;
            // fade out
            if let Some(frames) = life {
                if *frames < 10 {
                    color.a = *frames as f32 / 10.
                }
            }

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

        let frame_frac = self.control.frame_fraction();

        for snake_idx in 0..self.snakes.len() {
            let (snake, other_snakes) = Self::split_snakes(&mut self.snakes, snake_idx);

            // update the direction of the snake as soon as possible (mid game-frame)
            snake.update_dir(other_snakes, &self.apples, self.dim);

            let len = snake.len();

            // draw white aura around snake heads (debug)
            // for pos in snake.reachable(7, self.dim) {
            //     let dest = pos.to_point(self.cell_dim);
            //     let points = get_full_hexagon(dest, self.cell_dim);
            //     builder.polygon(DrawMode::fill(), &points, WHITE)?;
            // }

            let color_offset = match snake.state {
                SnakeState::Dying(offset) => offset,
                _ => 0,
            };

            for (seg_idx, segment) in snake.body.cells.iter().enumerate() {
                // previous = towards head
                // next = towards tail

                let previous = seg_idx
                    .checked_sub(1)
                    .map(|prev_idx| -snake.body.cells[prev_idx].next_segment)
                    .unwrap_or_else(|| snake.dir());

                let next = segment.next_segment;

                if seg_idx == 0 && matches!(snake.state, SnakeState::Crashed | SnakeState::Dying(_))
                {
                    assert!(
                        matches!(segment.typ, SegmentType::Crashed | SegmentType::BlackHole),
                        "head of type {:?} in snake in state {:?}",
                        segment.typ,
                        snake.state
                    );
                    // draw head separately
                    heads.push((snake_idx, *segment, previous, next, color_offset));
                    continue;
                }

                let dest = segment.pos.to_point(self.cell_dim);
                let color = snake
                    .painter
                    .paint_segment(seg_idx + color_offset, len, segment);

                let fraction = match seg_idx {
                    0 => SegmentFraction::Appearing(frame_frac),
                    i if i == len - 1 && snake.body.grow == 0 => {
                        if let SegmentType::Eaten {
                            original_food,
                            food_left,
                        } = segment.typ
                        {
                            let frac = ((original_food - food_left) as f32 + frame_frac)
                                / (original_food + 1) as f32;
                            SegmentFraction::Disappearing(frac)
                        } else {
                            SegmentFraction::Disappearing(frame_frac)
                        }
                    }
                    _ => SegmentFraction::Solid,
                };

                let points = if self.prefs.old_style_graphics {
                    get_full_hexagon(dest, self.cell_dim)
                } else {
                    get_points_animated(dest, previous, next, self.cell_dim, fraction)
                };

                builder.polygon(DrawMode::fill(), &points, color)?;
            }
        }

        for (snake_idx, segment, previous, next, color_offset) in heads {
            let Segment { pos, typ, .. } = segment;
            let dest = pos.to_point(self.cell_dim);
            let snake = &mut self.snakes[snake_idx];

            match typ {
                SegmentType::BlackHole => {
                    let hexagon_color = Color::from_rgb(1, 36, 92);
                    let segment_color = snake.painter.paint_segment(
                        color_offset,
                        snake.len() + color_offset,
                        &segment,
                    );
                    let hexagon_points = get_full_hexagon(dest, self.cell_dim);
                    let segment_points = if self.prefs.old_style_graphics {
                        get_full_hexagon(dest, self.cell_dim)
                    } else {
                        get_points_animated(
                            dest,
                            previous,
                            next,
                            self.cell_dim,
                            SegmentFraction::Appearing(0.5),
                        )
                    };
                    builder.polygon(DrawMode::fill(), &hexagon_points, hexagon_color)?;
                    builder.polygon(DrawMode::fill(), &segment_points, segment_color)?;
                }
                SegmentType::Crashed => {
                    let color = snake
                        .painter
                        .paint_segment(0, snake.len(), &snake.body.cells[0]);
                    let points = if self.prefs.old_style_graphics {
                        get_full_hexagon(dest, self.cell_dim)
                    } else {
                        get_points_animated(
                            dest,
                            previous,
                            next,
                            self.cell_dim,
                            SegmentFraction::Appearing(0.5),
                        )
                    };
                    builder.polygon(DrawMode::fill(), &points, color)?;
                }
                _ => unreachable!(
                    "head segment of type {:?} should not have been queued to be drawn separately",
                    typ
                ),
            }
        }

        for apple in &self.apples {
            let color = match apple.typ {
                AppleType::Normal(_) => self.palette.apple_color,
                AppleType::SpawnSnake(_) => {
                    let hue = 360. * (self.control.graphics_frame_num() as f64 / 60. % 1.);
                    let hsl = HSL {
                        h: hue,
                        s: 1.,
                        l: 0.3,
                    };
                    Color::from(hsl.to_rgb())
                }
            };
            if self.prefs.old_style_graphics {
                let dest = apple.pos.to_point(self.cell_dim);
                let points = get_full_hexagon(dest, self.cell_dim);
                builder.polygon(DrawMode::fill(), &points, color)?;
            } else {
                let dest = apple.pos.to_point(self.cell_dim) + self.cell_dim.center();
                builder.circle(DrawMode::fill(), dest, self.cell_dim.side / 1.5, 0.1, color);
            }
        }

        let mesh = builder.build(ctx)?;
        draw(ctx, &mesh, DrawParam::default())
    }
}

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        while self.control.can_update() {
            self.advance_snakes();
            self.spawn_apples();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.control.graphics_frame();

        if self.prefs.display_fps {
            let game_fps = self.control.measured_game_fps();
            let graphics_fps = self.control.measured_graphics_fps();

            let game_fps_undershoot = (self.control.fps() as f64 - game_fps) / game_fps;
            let graphics_fps_undershoot = (60. - graphics_fps) / graphics_fps;
            let color = if game_fps_undershoot > 0.05 || graphics_fps_undershoot > 0.05 {
                // > 5% undershoot: red
                Color::from_rgb(200, 0, 0)
            } else if game_fps_undershoot > 0.02 || graphics_fps_undershoot > 0.02 {
                // > 2% undershoot: orange
                Color::from_rgb(235, 168, 52)
            } else {
                // at or overshoot: white
                WHITE
            };

            self.message_top_left = Some(Message::from((
                format!("u: {:.2} g: {:.2}", game_fps, graphics_fps),
                color,
            )));
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

        // TODO: also tie these to a keymap (dvorak-centric for now)
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
                self.message_top_right = Some(Message::from((
                    message.to_string(),
                    self.prefs.message_duration,
                )));
            }
            F => {
                self.prefs.display_fps = !self.prefs.display_fps;
                if !self.prefs.display_fps {
                    self.message_top_left = None;
                }
            }
            A => {
                // only apply if there is exactly one player snake
                if self.seeds.len() == 1 {
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
                                let mut controller = STASHED_CONTROLLER.take().unwrap();
                                controller.reset(player_snake.body.dir);
                                player_snake.controller = controller;
                                message = "Autopilot off"
                            }
                        }

                        self.message_top_right = Some(Message::from((
                            message.to_string(),
                            self.prefs.message_duration,
                        )));
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
                self.message_top_right = Some(Message::from((
                    format!("fps: {}", new_fps),
                    self.prefs.message_duration,
                )));
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
                self.message_top_right = Some(Message::from((
                    format!("fps: {}", new_fps),
                    self.prefs.message_duration,
                )));
            }
            Escape => {
                self.prefs.old_style_graphics = !self.prefs.old_style_graphics;
                let message = if self.prefs.old_style_graphics {
                    "RTX off"
                } else {
                    "RTX on"
                };
                self.message_top_right = Some(Message::from((
                    message.to_string(),
                    self.prefs.message_duration,
                )));
            }
            X => {
                self.prefs.special_apples = !self.prefs.special_apples;
                let message = if self.prefs.special_apples {
                    "Special apples enabled"
                } else {
                    "Special apples disabled"
                };
                self.message_top_right = Some(Message::from((
                    message.to_string(),
                    self.prefs.message_duration,
                )));
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
                self.message_top_right = Some(Message::from((
                    format!("Apple food: {}", new_food),
                    self.prefs.message_duration,
                )));
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

                self.message_top_right = Some(Message::from((
                    format!("Cell side: {}", new_side),
                    self.prefs.message_duration,
                )));
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
        self.message_top_right = Some(Message::from((
            format!("{}x{}", h, v),
            self.prefs.message_duration,
        )));
    }
}
