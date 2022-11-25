use std::collections::HashMap;

use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::graphics::{
    Color, DrawMode, DrawParam, Mesh, MeshBuilder, {self},
};
use ggez::Context;
use rand::prelude::*;

use crate::app::distance_grid::DistanceGrid;
use crate::app::fps_control::{
    FpsControl, {self},
};
use crate::app::game_context::GameContext;
use crate::app::message::{Message, MessageID};
use crate::app::palette::Palette;
use crate::app::screen::board_dim::{calculate_board_dim, calculate_offset};
use crate::app::screen::Environment;
use crate::app::snake_management::{
    advance_snakes, find_collisions, handle_collisions, spawn_snakes,
};
use crate::app::stats::Stats;
use crate::app::{ guidance};
use crate::apple::spawn::{spawn_apples, SpawnPolicy};
use crate::apple::{
    Apple, {self},
};
use crate::basic::{CellDim, Dir, Food, HexDim, HexPoint, Point};
use crate::error::{AppErrorConversion, AppResult, Error};
use crate::rendering;
use crate::snake::{
    PassthroughKnowledge, Snake, {self},
};
use crate::snake_control::{
    Controller, {self},
};
use crate::support::flip::Flip;
use crate::support::row::ROw;
use crate::view::snakes::OtherSnakes;

pub struct Game {
    fps_control: FpsControl,

    gtx: GameContext,

    /// Offset to center the grid in the window
    offset: Point,

    seeds: Vec<snake::Builder>,
    snakes: Vec<Snake>,
    // TODO: keep apples in order of position to allow for binary search
    // TODO: specialized Vec for that
    apples: Vec<Apple>,

    rng: ThreadRng,

    distance_grid: DistanceGrid,

    /// These meshes are always cached and only
    /// recalculated when the board is resized
    grid_mesh: Option<Mesh>,
    border_mesh: Option<Mesh>,

    messages: HashMap<MessageID, Message>,

    /// Cached meshes used when the game is paused
    /// but redrawing still needs to occur (e.g. to
    /// display a message fade or animated apple)
    cached_snake_mesh: Option<Mesh>,
    cached_apple_mesh: Option<Mesh>,
    cached_distance_grid_mesh: Option<Mesh>,

    // TODO: move this mechanism to Control
    /// Consider the draw cache invalid for the
    /// next n frames, forces a redraw even if
    /// nothing changed, this is necessary to
    /// avoid visual glitches
    draw_cache_invalid: usize,

    guidance_path: Option<Vec<HexPoint>>,
}

impl Game {
    #[allow(dead_code)]
    pub fn new(
        cell_dim: CellDim,
        starting_fps: f64,
        seeds: Vec<snake::Builder>,
        palette: Palette,
        apple_spawn_policy: SpawnPolicy,
        ctx: &mut Context,
    ) -> Self {
        assert!(!seeds.is_empty(), "No players specified");

        let mut this = Self {
            fps_control: FpsControl::new(starting_fps),

            gtx: GameContext {
                // updated immediately after creation
                board_dim: HexPoint { h: 0, v: 0 },
                cell_dim,
                palette,
                prefs: Default::default(),
                apple_spawn_policy,
                frame_stamp: (0, 0.0),
                game_frame_num: 0,
                elapsed_millis: 0,
            },
            // updated immediately after creation
            offset: Point { x: 0., y: 0. },

            seeds: seeds.into_iter().map(Into::into).collect(),
            snakes: vec![],
            apples: vec![],

            rng: thread_rng(),

            distance_grid: DistanceGrid::new(),

            grid_mesh: None,
            border_mesh: None,

            messages: HashMap::new(),

            cached_snake_mesh: None,
            cached_apple_mesh: None,
            cached_distance_grid_mesh: None,

            draw_cache_invalid: 0,

            guidance_path: None,
        };
        this.update_dim(ctx);
        // warning: this spawns apples before there are any snakes
        this.restart();
        this
    }

    fn update_dim(&mut self, ctx: &Context) {
        let board_dim = calculate_board_dim(ctx, self.gtx.cell_dim);

        self.offset = calculate_offset(ctx, board_dim, self.gtx.cell_dim);

        if self.gtx.board_dim != board_dim {
            self.gtx.board_dim = board_dim;

            // restart if player snake head has left board limits
            if self
                .snakes
                .iter()
                .any(|s| s.snake_type == snake::Type::Player && !board_dim.contains(s.head().pos))
            {
                println!("warning: player snake outside of board, restarting");
                self.restart();
            } else {
                // remove snakes outside of board limits
                self.snakes
                    .retain(move |snake| board_dim.contains(snake.head().pos));

                // remove apples outside of board limits
                self.apples
                    .retain(move |apple| board_dim.contains(apple.pos));
                self.spawn_apples();
            }

            // invalidate
            self.grid_mesh = None;
            self.border_mesh = None;
            self.cached_apple_mesh = None;
            self.cached_snake_mesh = None;
            self.cached_distance_grid_mesh = None;
            self.distance_grid.invalidate();
        }
    }

    fn restart(&mut self) {
        self.snakes.clear();
        self.apples.clear();

        // seeds without a defined spawn point
        let unpositioned = self
            .seeds
            .iter()
            .filter(|seed| !matches!(seed.snake_type, Some(snake::Type::Simulated { .. })))
            .count();

        // TODO: clean this mess
        let mut unpositioned_dir = Dir::U;
        let mut unpositioned_h_pos: Box<dyn Iterator<Item = isize>> = if unpositioned > 0 {
            const DISTANCE_BETWEEN_SNAKES: isize = 1;

            let total_width = (unpositioned - 1) as isize * DISTANCE_BETWEEN_SNAKES + 1;
            assert!(total_width < self.gtx.board_dim.h, "snakes spread too wide");

            let half = total_width / 2;
            let middle = self.gtx.board_dim.h / 2;
            let start = middle - half;
            let end = start + total_width - 1;

            Box::new((start..=end).step_by(DISTANCE_BETWEEN_SNAKES as usize))
        } else {
            Box::new(std::iter::empty())
        };

        for seed in self.seeds.iter() {
            match seed.snake_type {
                Some(snake::Type::Simulated) => {
                    // expected to have initial position, direction, and length
                    self.snakes.push(seed.build().unwrap());
                }
                _ => {
                    self.snakes.push(
                        seed.clone()
                            .pos(HexPoint {
                                h: unpositioned_h_pos.next().unwrap(),
                                v: self.gtx.board_dim.v / 2,
                            })
                            .dir(unpositioned_dir)
                            .len(10)
                            .build()
                            .unwrap(),
                    );

                    // alternate
                    unpositioned_dir = -unpositioned_dir;
                }
            }
        }

        let left = unpositioned_h_pos.count();
        assert_eq!(left, 0, "unexpected iterator length");

        self.spawn_apples();
    }

    fn advance_snakes(&mut self, ctx: &Context) -> AppResult {
        advance_snakes(self, ctx);

        // if only ephemeral AIs are left, kill all other snakes
        let dying_or_ephemeral = |snake: &Snake| {
            matches!(snake.state, snake::State::Dying)
                || matches!(
                    snake.snake_type,
                    snake::Type::Competitor { life: Some(_) }
                        | snake::Type::Killer { life: Some(_) }
                )
        };
        if self.snakes.iter().all(dying_or_ephemeral) {
            for snake in &mut self.snakes {
                snake.die();
            }
        }

        if self.snakes.is_empty() {
            self.fps_control.game_over();
            self.draw_cache_invalid = 5;
            return Ok(());
        }

        let collisions = find_collisions(self);
        let (seeds, game_over) = handle_collisions(self, &collisions);

        if game_over {
            self.fps_control.game_over()
        }

        spawn_snakes(self, seeds)
            .map_err(Error::from)
            .with_trace_step("Game::advance_snakes")
    }
}

impl Game {
    /// Bounds for the length of one of the six sides of a cell
    const CELL_SIDE_MIN: f32 = 5.;
    const CELL_SIDE_MAX: f32 = 50.;

    fn spawn_apples(&mut self) {
        let new_apples = spawn_apples(&self.snakes, &self.apples, &mut self.gtx, &mut self.rng);
        self.apples.extend(new_apples.into_iter())
    }

    fn draw_messages(&mut self, ctx: &mut Context) -> AppResult {
        // draw messages and remove the ones that have
        // outlived their durations
        let remove = self
            .messages
            .iter()
            .map(|(id, message)| {
                message
                    .draw(ctx)
                    .map(|keep| if !keep { Some(*id) } else { None })
            })
            .collect::<AppResult<Vec<_>>>()
            .with_trace_step("draw_messages")?;
        remove
            .iter()
            .filter_map(|o| *o)
            .for_each(|id| drop(self.messages.remove(&id)));
        Ok(())
    }

    /// Display a notification message in the top-right
    /// corner with limited duration and default parameters,
    /// overwrite any previous notification message
    fn display_notification<S: ToString>(&mut self, text: S) {
        self.messages.insert(
            MessageID::Notification,
            Message::default_top_right(
                text.to_string(),
                Color::WHITE,
                Some(self.gtx.prefs.message_duration),
            ),
        );
    }

    /// Show game and graphics FPS information in the
    /// top-left corner
    fn update_fps_message(&mut self) {
        let game_fps = self.fps_control.measured_game_fps();
        let graphics_fps = self.fps_control.measured_graphics_fps();

        let game_fps_undershoot = (self.fps_control.fps() as f64 - game_fps) / game_fps;
        let graphics_fps_undershoot = (60. - graphics_fps) / graphics_fps;
        let color = if game_fps_undershoot > 0.05 || graphics_fps_undershoot > 0.05 {
            // > 5% undershoot: red
            Color::from_rgb(200, 0, 0)
        } else if game_fps_undershoot > 0.02 || graphics_fps_undershoot > 0.02 {
            // > 2% undershoot: orange
            Color::from_rgb(235, 168, 52)
        } else {
            // at or overshoot: white
            Color::WHITE
        };

        self.messages.insert(
            MessageID::Fps,
            Message::default_top_left(
                format!("u: {:.2} g: {:.2}", game_fps, graphics_fps),
                color,
                None,
            ),
        );
    }

    fn first_player_snake_idx(&self) -> Option<usize> {
        self.snakes
            .iter()
            .position(|snake| snake.snake_type == snake::Type::Player)
    }
}

impl EventHandler<Error> for Game {
    fn update(&mut self, ctx: &mut Context) -> AppResult {
        while self.fps_control.can_update(&mut self.gtx) {
            self.advance_snakes(ctx).with_trace_step("Game::update")?;
            self.spawn_apples();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> AppResult {
        self.fps_control.graphics_frame(&mut self.gtx);

        if self.gtx.prefs.display_fps {
            self.update_fps_message();
        }

        // TODO: diagnose why the interframe interval is
        //  1ms sometimes when out of focus
        // unsafe {
        //     use std::time::Instant;
        //     static mut L: Option<Instant> = None;
        //     if let Some(last) = L {
        //         println!("{}ms", last.elapsed().as_millis());
        //     }
        //     L = Some(Instant::now());
        // }

        // TODO: fix this mess
        // TODO: reintroduce a short grace period after
        //  game over for all the graphics to properly update
        // selectively set to Some(_) if they need to be updated
        let mut grid_mesh = None;
        let mut border_mesh = None;
        let mut apple_mesh = None;
        let mut snake_mesh = None;
        let mut distance_grid_mesh = None;

        let mut stats = Stats::default();

        if self.fps_control.state() == fps_control::State::Playing {
            // Update the direction of the snake early
            // to see it turning as soon as possible,
            // this could happen in the middle of a
            // game frame. Repeated updates during the
            // same game frame are blocked
            for idx in 0..self.snakes.len() {
                let (snake, other_snakes) = OtherSnakes::split_snakes(&mut self.snakes, idx);
                snake.update_dir(other_snakes, &self.apples, &self.gtx, ctx);
            }

            self.cached_snake_mesh = None;
            self.cached_apple_mesh = None;
            self.cached_distance_grid_mesh = None;

            // TODO: refactor this utter mess of a code
            apple_mesh = Some(ROw::Owned(rendering::apple_mesh(
                &self.apples,
                &self.gtx,
                ctx,
                &mut stats,
            )?));
            snake_mesh = Some(ROw::Owned(rendering::snake_mesh(
                &mut self.snakes,
                &self.gtx,
                ctx,
                &mut stats,
            )?));

            if self.gtx.prefs.draw_grid {
                if self.grid_mesh.is_none() {
                    self.grid_mesh = Some(rendering::grid_mesh(&self.gtx, ctx)?);
                };
                grid_mesh = Some(self.grid_mesh.as_ref().unwrap());
            }
            if self.gtx.prefs.draw_border {
                if self.border_mesh.is_none() {
                    self.border_mesh = Some(rendering::border_mesh(&self.gtx, ctx)?);
                }
                border_mesh = Some(self.border_mesh.as_ref().unwrap());
            }
            if self.gtx.prefs.draw_distance_grid {
                distance_grid_mesh = Some(ROw::Owned({
                    // draw colored grid of distances from player snake head
                    let player_snake_idx = self
                        .snakes
                        .iter()
                        .position(|snake| snake.snake_type == snake::Type::Player)
                        .expect("no player snake");
                    let (player_snake, other_snakes) =
                        OtherSnakes::split_snakes(&mut self.snakes, player_snake_idx);

                    self.distance_grid
                        .mesh(player_snake, other_snakes, ctx, &self.gtx)?
                }));
            }
        } else {
            let mut update = false;

            // update apples if there are any animated ones
            if self.cached_apple_mesh.is_none()
                || self
                    .apples
                    .iter()
                    .any(|apple| matches!(apple.apple_type, apple::Type::SpawnSnake(_)))
            {
                self.cached_apple_mesh = Some(rendering::apple_mesh(
                    &self.apples,
                    &self.gtx,
                    ctx,
                    &mut stats,
                )?);
                update = true;
            }

            if self.cached_snake_mesh.is_none() {
                self.cached_snake_mesh = Some(rendering::snake_mesh(
                    &mut self.snakes,
                    &self.gtx,
                    ctx,
                    &mut stats,
                )?);
                update = true;
            }

            if self.cached_distance_grid_mesh.is_none() {
                // draw colored grid of distances from player snake head
                let player_snake_idx = self
                    .snakes
                    .iter()
                    .position(|snake| snake.snake_type == snake::Type::Player)
                    .expect("no player snake");
                let (player_snake, other_snakes) =
                    OtherSnakes::split_snakes(&mut self.snakes, player_snake_idx);

                self.cached_distance_grid_mesh =
                    Some(
                        self.distance_grid
                            .mesh(player_snake, other_snakes, ctx, &self.gtx)?,
                    );
            }

            if !self.messages.is_empty() {
                update = true;
            }

            if self.draw_cache_invalid > 0 {
                self.draw_cache_invalid -= 1;
                update = true;
            }

            if update {
                if self.gtx.prefs.draw_grid {
                    if self.grid_mesh.is_none() {
                        self.grid_mesh = Some(rendering::grid_mesh(&self.gtx, ctx)?);
                    };
                    grid_mesh = Some(self.grid_mesh.as_ref().unwrap());
                }
                if self.gtx.prefs.draw_border {
                    if self.border_mesh.is_none() {
                        self.border_mesh = Some(rendering::border_mesh(&self.gtx, ctx)?);
                    }
                    border_mesh = Some(self.border_mesh.as_ref().unwrap());
                }
                if self.gtx.prefs.draw_distance_grid {
                    distance_grid_mesh =
                        Some(ROw::Ref(self.cached_distance_grid_mesh.as_ref().unwrap()));
                }
                apple_mesh = Some(ROw::Ref(self.cached_apple_mesh.as_ref().unwrap()));
                snake_mesh = Some(ROw::Ref(self.cached_snake_mesh.as_ref().unwrap()));
            }
        }

        let draw_param = DrawParam::default().dest(self.offset);

        if grid_mesh.is_some()
            || border_mesh.is_some()
            || apple_mesh.is_some()
            || snake_mesh.is_some()
            || distance_grid_mesh.is_some()
        {
            graphics::clear(ctx, self.gtx.palette.background_color);

            if let Some(mesh) = distance_grid_mesh {
                graphics::draw(ctx, mesh.get(), draw_param)?;
            }

            if let Some(mesh) = grid_mesh {
                graphics::draw(ctx, mesh, draw_param)?;
            }
            {
                let idx = self.first_player_snake_idx().unwrap();
                let (player_snake, other_snakes) = OtherSnakes::split_snakes(&mut self.snakes, idx);

                let recalculate = match &mut self.guidance_path {
                    None => true,
                    Some(path) if path.is_empty() => true,
                    Some(path) if path.get(1) == Some(&player_snake.head().pos) => {
                        // following existing path, just pop
                        path.remove(0);
                        false
                    }
                    Some(path) if path[0] != player_snake.head().pos => true,
                    _ => false,
                };

                if recalculate {
                    self.guidance_path = Some(guidance::get_guidance_path(
                        player_snake,
                        other_snakes,
                        &self.apples,
                        ctx,
                        &self.gtx,
                    ))
                }

                let mut builder = MeshBuilder::new();
                for pos in self.guidance_path.as_ref().unwrap() {
                    let dest = pos.to_cartesian(self.gtx.cell_dim) + self.gtx.cell_dim.center();
                    builder.circle(
                        DrawMode::fill(),
                        dest,
                        self.gtx.cell_dim.side / 2.5,
                        0.1,
                        Color::WHITE,
                    )?;
                }
                let mesh = builder.build(ctx)?;
                graphics::draw(ctx, &mesh, draw_param)?;
            }
            if let Some(mesh) = snake_mesh {
                graphics::draw(ctx, mesh.get(), draw_param)?;
            }
            if let Some(mesh) = apple_mesh {
                graphics::draw(ctx, mesh.get(), draw_param)?;
            }
            if let Some(mesh) = border_mesh {
                graphics::draw(ctx, mesh, draw_param)?;
            }
            // only draw snake_control artifacts for player snake(s)
            for snake in &self.snakes {
                if snake.snake_type == snake::Type::Player {
                    if let Some(mesh) = snake.controller.get_mesh(&self.gtx, ctx) {
                        graphics::draw(ctx, &mesh?, draw_param)?;
                    }
                }
            }

            if self.gtx.prefs.display_stats {
                let message = stats.get_stats_message();
                self.messages.insert(MessageID::Stats, message);
            }

            self.draw_messages(ctx)?;
        }

        graphics::present(ctx)
            .map_err(Error::from)
            .with_trace_step("Game::draw")
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
        use KeyCode::*;

        let numeric_keys = [Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9];

        // TODO: also tie these to a keymap (dvorak-centric for now)
        match key {
            Space => match self.fps_control.state() {
                fps_control::State::GameOver => {
                    self.restart();
                    self.fps_control.play();
                }
                fps_control::State::Playing => {
                    self.fps_control.pause();
                    self.draw_cache_invalid = 5;
                }
                fps_control::State::Paused => self.fps_control.play(),
            },
            G => {
                self.gtx.prefs.draw_border = self.gtx.prefs.draw_grid.flip();
                let text = if self.gtx.prefs.draw_grid {
                    "Grid on"
                } else {
                    "Grid off"
                };
                self.display_notification(text);
            }
            D => {
                let text = if self.gtx.prefs.draw_distance_grid.flip() {
                    "Distance grid on"
                } else {
                    "Distance grid off"
                };
                self.display_notification(text);
            }
            F => {
                if !self.gtx.prefs.display_fps.flip() {
                    self.messages.remove(&MessageID::Fps);
                    self.draw_cache_invalid = 5;
                }
            }
            S => {
                if !self.gtx.prefs.display_stats.flip() {
                    self.messages.remove(&MessageID::Stats);
                    self.draw_cache_invalid = 5;
                }
            }
            Y => {
                if !self.gtx.prefs.draw_ai_debug_artifacts.flip() {
                    for snake in &mut self.snakes {
                        snake.ai_artifacts = None;
                    }
                }
            }
            A => {
                // only apply if there is exactly one player snake
                if self.seeds.len() == 1 {
                    // WARNING: hacky
                    unsafe {
                        static mut STASHED_CONTROLLER: Option<Box<dyn Controller + Send + Sync>> =
                            None;

                        let player_snake = self
                            .snakes
                            .iter_mut()
                            .find(|snake| snake.snake_type == snake::Type::Player)
                            .unwrap();

                        let text = match &STASHED_CONTROLLER {
                            None => {
                                STASHED_CONTROLLER = Some(std::mem::replace(
                                    &mut player_snake.controller,
                                    snake_control::Template::AStar {
                                        passthrough_knowledge: PassthroughKnowledge::accurate(
                                            &player_snake.eat_mechanics,
                                        ),
                                    }
                                    .into_controller(player_snake.body.dir),
                                ));
                                "Autopilot on"
                            }
                            Some(_) => {
                                let mut controller = STASHED_CONTROLLER.take().unwrap();
                                controller.reset(player_snake.body.dir);
                                player_snake.controller = controller;
                                "Autopilot off"
                            }
                        };
                        self.display_notification(text);
                    }
                }
            }
            LBracket => {
                let mut new_fps = match self.fps_control.fps() {
                    f if f <= 0.2 => 0.1,
                    f if f <= 1. => f - 0.1,
                    f if f <= 20. => f - 1.,
                    f if f <= 50. => f - 5.,
                    f if f <= 100. => f - 10.,
                    f if f <= 500. => f - 50.,
                    f if f <= 1000. => f - 100.,
                    f if f <= 10_000. => f - 1000.,
                    f => f - 10_000.,
                };
                new_fps = (new_fps * 10.).round() / 10.;

                self.fps_control.set_game_fps(new_fps);
                self.display_notification(format!("fps: {}", new_fps));
            }
            RBracket => {
                let mut new_fps = match self.fps_control.fps() {
                    f if f <= 0.1 => 0.2,
                    f if f < 1. => f + 0.1,
                    f if f < 20. => (f + 1.).floor(),
                    f if f < 50. => f + 5.,
                    f if f < 100. => f + 10.,
                    f if f < 500. => f + 50.,
                    f if f < 1000. => f + 100.,
                    f if f < 10_000. => f + 1000.,
                    f => f + 10_000.,
                };
                new_fps = (new_fps * 10.).round() / 10.;

                self.fps_control.set_game_fps(new_fps);
                self.display_notification(format!("fps: {}", new_fps));
            }
            Escape => {
                let text;
                match self.gtx.prefs.draw_style {
                    rendering::Style::Hexagon => {
                        self.gtx.prefs.draw_style = rendering::Style::Smooth;
                        text = "draw style: smooth";
                    }
                    rendering::Style::Smooth => {
                        self.gtx.prefs.draw_style = rendering::Style::Hexagon;
                        text = "draw style: hexagon";
                    }
                }
                self.display_notification(text);
                if self.fps_control.state() != fps_control::State::Playing {
                    self.draw_cache_invalid = 5;
                    self.cached_snake_mesh = None;
                    self.cached_apple_mesh = None;
                }
            }
            X => {
                let text = if self.gtx.prefs.special_apples.flip() {
                    "Special apples enabled"
                } else {
                    // replace special apples with normal apples
                    let apple_food = self.gtx.prefs.apple_food;
                    self.apples.iter_mut().for_each(|apple| {
                        if !matches!(apple.apple_type, apple::Type::Food(_)) {
                            *apple = Apple {
                                pos: apple.pos,
                                apple_type: apple::Type::Food(apple_food),
                            }
                        }
                    });
                    self.cached_apple_mesh = None;
                    "Special apples disabled"
                };
                self.display_notification(text);
            }
            #[rustfmt::skip] // rustfmt doesn't know about if let guards
            k if let Some(idx) = numeric_keys
                .iter()
                .position(|nk| *nk == k) =>
            {
                let new_food = idx as Food + 1;
                self.gtx.prefs.apple_food = new_food;
                // change existing apples
                for apple in & mut self.apples {
                    if let apple::Type::Food(food) = &mut apple.apple_type {
                        *food = new_food;
                    }
                }
                self.display_notification(format!("Apple food: {}", new_food));
            }
            k @ Down | k @ Up => {
                let factor = if k == Down { 0.9 } else { 1. / 0.9 };
                let mut new_side_length = self.gtx.cell_dim.side * factor;
                if new_side_length < Self::CELL_SIDE_MIN {
                    new_side_length = Self::CELL_SIDE_MIN
                } else if new_side_length > Self::CELL_SIDE_MAX {
                    new_side_length = Self::CELL_SIDE_MAX
                }
                self.gtx.cell_dim = CellDim::from(new_side_length);
                self.update_dim(ctx);
                self.display_notification(format!("Cell side: {}", new_side_length));
            }
            k => {
                if self.fps_control.state() == fps_control::State::Playing {
                    for snake in &mut self.snakes {
                        snake.controller.key_pressed(k)
                    }
                }
            }
        }
    }

    // TODO: forbid resizing in-game
    fn resize_event(&mut self, ctx: &mut Context, _width: f32, _height: f32) {
        self.update_dim(ctx);
        let HexDim { h, v } = self.gtx.board_dim;
        self.display_notification(format!("{}x{}", h, v));
    }
}

impl Environment for Game {
    fn snakes(&self) -> &[Snake] {
        &self.snakes
    }

    fn apples(&self) -> &[Apple] {
        &self.apples
    }

    fn snakes_apples_gtx_mut(&mut self) -> (&mut [Snake], &mut [Apple], &mut GameContext) {
        (&mut self.snakes, &mut self.apples, &mut self.gtx)
    }

    fn snakes_apples_rng_mut(&mut self) -> (&mut [Snake], &mut [Apple], &mut ThreadRng) {
        (&mut self.snakes, &mut self.apples, &mut self.rng)
    }

    fn add_snake(&mut self, snake_builder: &snake::Builder) -> AppResult {
        self.snakes.push(
            snake_builder
                .build()
                .map_err(Error::from)
                .with_trace_step("Game::add_snake")?,
        );
        Ok(())
    }

    fn remove_snake(&mut self, index: usize) -> Snake {
        self.snakes.remove(index)
    }

    fn remove_apple(&mut self, index: usize) -> Apple {
        let ret = self.apples.remove(index);
        self.guidance_path = None; // invalidate
        ret
    }

    fn gtx(&self) -> &GameContext {
        &self.gtx
    }

    fn rng(&mut self) -> &mut ThreadRng {
        &mut self.rng
    }
}
