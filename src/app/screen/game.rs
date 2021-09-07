use std::collections::HashMap;

use ggez::{
    conf::WindowMode,
    Context,
    event::{EventHandler, KeyCode, KeyMods},
    GameResult, graphics::{clear, Color, draw, DrawParam, Mesh, present},
};
use rand::prelude::*;

use crate::{
    app::{
        collisions::{find_collisions, handle_collisions},
        palette::Palette,
        screen::{
            rendering::{
                apple_mesh::get_apple_mesh,
                grid_mesh::{get_border_mesh, get_grid_mesh},
                snake_mesh::get_snake_mesh,
            },
        },
        snake::{
            self,
            controller::{Controller, ControllerTemplate},
            EatBehavior,
            EatMechanics, Seed, Snake, State, Type, utils::split_snakes_mut,
        },
    },
    basic::{CellDim, Dir, DrawStyle, HexDim, HexPoint, Point},
};
use crate::app::apple::{self, Apple};
use crate::app::apple::spawning::{ScheduledSpawn, spawn_apples, SpawnPolicy};
use crate::app::control::{Control, GameState};
use crate::app::message::{Message, MessageID};
use crate::app::prefs::{ Prefs};
use crate::app::{stats::Stats, utils::Food};
use crate::row::ROw;
use crate::app::utils::{get_occupied_cells, random_free_spot};
use crate::app::collisions::spawn_snakes;

pub struct Game {
    control: Control,

    /// Dimension of the window
    window_dim: Point,
    /// Hex-dimension of the grid
    board_dim: HexDim,
    /// Offset to center the grid in the window
    offset: Point,

    seeds: Vec<Seed>,
    snakes: Vec<Snake>,
    apples: Vec<Apple>,

    apple_spawn_policy: SpawnPolicy,

    cell_dim: CellDim,
    palette: Palette,

    rng: ThreadRng,

    /// These meshes are always cached and only
    /// recalculated when the board is resized
    grid_mesh: Option<Mesh>,
    border_mesh: Option<Mesh>,

    prefs: Prefs,
    messages: HashMap<MessageID, Message>,

    /// Cached meshes used when the game is paused
    /// but redrawing still needs to occur (e.g. to
    /// display a message fade or animated apple)
    cached_snake_mesh: Option<Mesh>,
    cached_apple_mesh: Option<Mesh>,

    /// Consider the draw cache invalid for the
    /// next n frames, forces a redraw even if
    /// nothing changed, this is necessary to
    /// avoid visual glitches
    draw_cache_invalid: usize,
}

impl Game {
    pub fn new(
        cell_dim: CellDim,
        starting_fps: f64,
        seeds: Vec<Seed>,
        palette: Palette,
        apple_spawn_policy: SpawnPolicy,
        wm: WindowMode,
    ) -> Self {
        assert!(!seeds.is_empty(), "No players specified");

        let mut game = Self {
            control: Control::new(starting_fps),

            window_dim: Point { x: wm.width, y: wm.height },
            // board_dim and offset get updated immediately after creation
            // by calling update_dim()
            board_dim: HexDim { h: 0, v: 0 },
            offset: Point { x: 0., y: 0. },

            seeds: seeds.into_iter().map(Into::into).collect(),
            snakes: vec![],
            apples: vec![],

            apple_spawn_policy,

            cell_dim,
            palette,

            rng: thread_rng(),
            grid_mesh: None,
            border_mesh: None,

            prefs: Prefs::default(),
            messages: HashMap::new(),

            cached_snake_mesh: None,
            cached_apple_mesh: None,

            draw_cache_invalid: 0,
        };
        // warning: this spawns apples before there are any snakes
        game.update_dim();
        game.restart();
        game
    }

    fn update_dim(&mut self) {
        let Point { x: width, y: height } = self.window_dim;
        let CellDim { side, sin, cos } = self.cell_dim;
        let new_board_dim = HexDim {
            h: ((width - cos) / (side + cos)) as isize,
            v: ((height - sin) / (2. * sin)) as isize,
        };

        if self.board_dim != new_board_dim {
            self.board_dim = new_board_dim;

            // restart if player snake head has left board limits
            if self
                .snakes
                .iter()
                .any(|s| s.snake_type == Type::Player && !new_board_dim.contains(s.head().pos))
            {
                println!("warning: player snake outside of board, restarting");
                self.restart();
            } else {
                // remove snakes outside of board limits
                self.snakes
                    .retain(move |snake| new_board_dim.contains(snake.head().pos));

                // remove apples outside of board limits
                self.apples.retain(move |apple| new_board_dim.contains(apple.pos));
                self.spawn_apples();
            }

            // invalidate
            self.grid_mesh = None;
            self.border_mesh = None;
            self.cached_apple_mesh = None;
            self.cached_snake_mesh = None;
        }

        let board_cartesian_dim = Point {
            x: new_board_dim.h as f32 * (side + cos) + cos,
            y: new_board_dim.v as f32 * 2. * sin + sin,
        };
        self.offset = (self.window_dim - board_cartesian_dim) / 2.;
    }

    fn restart(&mut self) {
        self.snakes.clear();
        self.apples.clear();

        // seeds without a defined spawn point
        let unpositioned = self
            .seeds
            .iter()
            .filter(|seed| !matches!(seed.snake_type, Type::Simulated { .. }))
            .count();

        let mut unpositioned_dir = Dir::U;
        let mut unpositioned_h_pos: Box<dyn Iterator<Item = isize>> = if unpositioned > 0 {
            const DISTANCE_BETWEEN_SNAKES: isize = 1;

            let total_width = (unpositioned - 1) as isize * DISTANCE_BETWEEN_SNAKES + 1;
            assert!(total_width < self.board_dim.h, "snakes spread too wide");

            let half = total_width / 2;
            let middle = self.board_dim.h / 2;
            let start = middle - half;
            let end = start + total_width - 1;

            Box::new((start..=end).step_by(DISTANCE_BETWEEN_SNAKES as usize))
        } else {
            Box::new(std::iter::empty())
        };

        for seed in self.seeds.iter() {
            match seed.snake_type {
                Type::Simulated { start_pos, start_dir, start_grow } => {
                    self.snakes
                        .push(Snake::from_seed(seed, start_pos, start_dir, start_grow));
                }
                _ => {
                    self.snakes.push(Snake::from_seed(
                        seed,
                        HexPoint {
                            h: unpositioned_h_pos.next().unwrap(),
                            v: self.board_dim.v / 2,
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

    fn advance_snakes(&mut self) {
        let mut remove_snakes = vec![];
        for snake_idx in 0..self.snakes.len() {
            // set snake to die if it ran out of life
            match &mut self.snakes[snake_idx].snake_type {
                Type::Competitor { life: Some(life) }
                | Type::Killer { life: Some(life) } => {
                    if *life == 0 {
                        self.snakes[snake_idx].die();
                    } else {
                        *life -= 1;
                    }
                }
                _ => (),
            }

            let (snake, other_snakes) = split_snakes_mut(&mut self.snakes, snake_idx);

            // advance the snake
            snake.advance(
                other_snakes,
                &self.apples,
                self.board_dim,
                self.control.frame_stamp(),
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
            matches!(snake.state, State::Dying)
                || matches!(
                    snake.snake_type,
                    Type::Competitor { life: Some(_) } | Type::Killer { life: Some(_) }
                )
        };
        if self.snakes.iter().all(dying_or_ephemeral) {
            for snake in &mut self.snakes {
                snake.die();
            }
        }

        if self.snakes.is_empty() {
            self.control.game_over();
            self.draw_cache_invalid = 5;
            return;
        }

        let collisions = find_collisions(&self.snakes, &self.apples);
        let (seeds, remove_apples, game_over) =
            handle_collisions(&collisions, &mut self.snakes, &self.apples);

        if game_over {
            self.control.game_over()
        }

        spawn_snakes(seeds, &mut self.snakes, &self.apples, self.board_dim, &mut self.rng);

        for apple_index in remove_apples.into_iter().rev() {
            self.apples.remove(apple_index);
        }
    }
}

impl Game {
    /// Bounds for the length of one of the six sides of a cell
    const CELL_SIDE_MIN: f32 = 5.;
    const CELL_SIDE_MAX: f32 = 50.;

    fn spawn_apples(&mut self) {
        let new_apples = spawn_apples(&mut self.apple_spawn_policy, self.board_dim, &self.snakes, &self.apples, &self.prefs, &mut self.rng);
        self.apples.extend(new_apples.into_iter())
    }

    fn draw_messages(&mut self, ctx: &mut Context) -> GameResult {
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
            .collect::<GameResult<Vec<_>>>()?;
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
                Some(self.prefs.message_duration),
            ),
        );
    }

    /// Show game and graphics FPS information in the
    /// top-left corner
    fn update_fps_message(&mut self) {
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
}

impl EventHandler<ggez::GameError> for Game {
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

        // TODO: fix this mess, reintroduce a short grace period after
        //  game over for all the graphics to properly update
        // selectively set to Some(_) if they need to be updated
        let mut grid_mesh = None;
        let mut border_mesh = None;
        let mut snake_mesh = None;
        let mut apple_mesh = None;

        let frame_stamp = self.control.frame_stamp();
        let mut stats = Stats::default();

        if self.control.state() == GameState::Playing {
            // Update the direction of the snake early
            // to see it turning as soon as possible,
            // this could happen in the middle of a
            // game frame. Repeated updates during the
            // same game frame are blocked
            for idx in 0..self.snakes.len() {
                let (snake, other_snakes) = split_snakes_mut(&mut self.snakes, idx);
                snake.update_dir(other_snakes, &self.apples, self.board_dim, frame_stamp);
            }

            self.cached_snake_mesh = None;
            self.cached_apple_mesh = None;

            snake_mesh = Some(ROw::Owned(get_snake_mesh(
                &mut self.snakes,
                frame_stamp,
                self.board_dim,
                self.cell_dim,
                self.prefs.draw_style,
                ctx,
                &mut stats,
            )?));
            apple_mesh = Some(ROw::Owned(get_apple_mesh(
                &self.apples,
                frame_stamp,
                self.cell_dim,
                self.prefs.draw_style,
                &self.palette,
                ctx,
                &mut stats,
            )?));
            if self.prefs.draw_grid {
                if self.grid_mesh.is_none() {
                    self.grid_mesh =
                        Some(get_grid_mesh(self.board_dim, self.cell_dim, &self.palette, ctx)?);
                };
                grid_mesh = Some(self.grid_mesh.as_ref().unwrap());
            }
            if self.prefs.draw_border {
                if self.border_mesh.is_none() {
                    self.border_mesh = Some(get_border_mesh(
                        self.board_dim,
                        self.cell_dim,
                        &self.palette,
                        ctx,
                    )?);
                }
                border_mesh = Some(self.border_mesh.as_ref().unwrap());
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
                self.cached_apple_mesh = Some(get_apple_mesh(
                    &self.apples,
                    frame_stamp,
                    self.cell_dim,
                    self.prefs.draw_style,
                    &self.palette,
                    ctx,
                    &mut stats,
                )?);
                update = true;
            }

            if self.cached_snake_mesh.is_none() {
                self.cached_snake_mesh = Some(get_snake_mesh(
                    &mut self.snakes,
                    frame_stamp,
                    self.board_dim,
                    self.cell_dim,
                    self.prefs.draw_style,
                    ctx,
                    &mut stats,
                )?);
                update = true;
            }

            if !self.messages.is_empty() {
                update = true;
            }

            if self.draw_cache_invalid > 0 {
                self.draw_cache_invalid -= 1;
                update = true;
            }

            if update {
                if self.prefs.draw_grid {
                    if self.grid_mesh.is_none() {
                        self.grid_mesh =
                            Some(get_grid_mesh(self.board_dim, self.cell_dim, &self.palette, ctx)?);
                    };
                    grid_mesh = Some(self.grid_mesh.as_ref().unwrap());
                }
                if self.prefs.draw_border {
                    if self.border_mesh.is_none() {
                        self.border_mesh = Some(get_border_mesh(
                            self.board_dim,
                            self.cell_dim,
                            &self.palette,
                            ctx,
                        )?);
                    }
                    border_mesh = Some(self.border_mesh.as_ref().unwrap());
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
        {
            clear(ctx, self.palette.background_color);

            if let Some(mesh) = grid_mesh {
                draw(ctx, mesh, draw_param)?;
            }
            if let Some(mesh) = snake_mesh {
                draw(ctx, mesh.get(), draw_param)?;
            }
            if let Some(mesh) = apple_mesh {
                draw(ctx, mesh.get(), draw_param)?;
            }
            if let Some(mesh) = border_mesh {
                draw(ctx, mesh, draw_param)?;
            }

            if self.prefs.display_stats {
                let message = stats.get_stats_message();
                self.messages.insert(MessageID::Stats, message);
            }

            self.draw_messages(ctx)?;
        }

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
                GameState::Playing => {
                    self.control.pause();
                    self.draw_cache_invalid = 5;
                },
                GameState::Paused => self.control.play(),
            },
            G => {
                self.prefs.draw_grid = !self.prefs.draw_grid;
                let text = if self.prefs.draw_grid {
                    "Grid on"
                } else {
                    "Grid off"
                };
                self.display_notification(text);
            }
            F => {
                self.prefs.display_fps = !self.prefs.display_fps;
                if !self.prefs.display_fps {
                    self.messages.remove(&MessageID::Fps);
                    self.draw_cache_invalid = 5;
                }
            }
            S => {
                self.prefs.display_stats = !self.prefs.display_stats;
                if !self.prefs.display_stats {
                    self.messages.remove(&MessageID::Stats);
                    self.draw_cache_invalid = 5;
                }
            }
            A => {
                // only apply if there is exactly one player snake
                if self.seeds.len() == 1 {
                    // hacky
                    unsafe {
                        static mut STASHED_CONTROLLER: Option<Box<dyn Controller>> = None;

                        let player_snake = self
                            .snakes
                            .iter_mut()
                            .find(|snake| snake.snake_type == Type::Player)
                            .unwrap();

                        let text = match &STASHED_CONTROLLER {
                            None => {
                                STASHED_CONTROLLER = Some(std::mem::replace(
                                    &mut player_snake.controller,
                                    ControllerTemplate::AStar
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
                let mut new_fps = match self.control.fps() {
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

                self.control.set_game_fps(new_fps);
                self.display_notification(format!("fps: {}", new_fps));
            }
            RBracket => {
                let mut new_fps = match self.control.fps() {
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

                self.control.set_game_fps(new_fps);
                self.display_notification(format!("fps: {}", new_fps));
            }
            Escape => {
                let text;
                match self.prefs.draw_style {
                    DrawStyle::Hexagon => {
                        self.prefs.draw_style = DrawStyle::Smooth;
                        text = "draw style: smooth";
                    }
                    DrawStyle::Smooth => {
                        self.prefs.draw_style = DrawStyle::Hexagon;
                        text = "draw style: hexagon";
                    }
                }
                self.display_notification(text);
                if self.control.state() != GameState::Playing {
                    self.draw_cache_invalid = 5;
                    self.cached_snake_mesh = None;
                    self.cached_apple_mesh = None;
                }
            }
            X => {
                self.prefs.special_apples = !self.prefs.special_apples;
                let text = if self.prefs.special_apples {
                    "Special apples enabled"
                } else {
                    // replace special apples with normal apples
                    let apple_food = self.prefs.apple_food;
                    self.apples.iter_mut().for_each(|apple| {
                        if !matches!(apple.apple_type, apple::Type::Normal(_)) {
                            *apple = Apple {
                                pos: apple.pos,
                                apple_type: apple::Type::Normal(apple_food),
                            }
                        }
                    });
                    self.cached_apple_mesh = None;
                    "Special apples disabled"
                };
                self.display_notification(text);
            }
            k if let Some(idx) = numeric_keys.iter().position(|nk| *nk == k) => {
            let new_food = idx as Food + 1;
            self.prefs.apple_food = new_food;
            // change existing apples
            for apple in & mut self.apples {
            if let apple::Type::Normal(food) = & mut apple.apple_type {
            * food = new_food;
            }
            }
            self.display_notification(format ! ("Apple food: {}", new_food));
            }
            k @ Down | k @ Up => {
                let factor = if k == Down { 0.9 } else { 1. / 0.9 };
                let mut new_side_length = self.cell_dim.side * factor;
                if new_side_length < Self::CELL_SIDE_MIN {
                    new_side_length = Self::CELL_SIDE_MIN
                } else if new_side_length > Self::CELL_SIDE_MAX {
                    new_side_length = Self::CELL_SIDE_MAX
                }
                self.cell_dim = CellDim::from(new_side_length);
                self.update_dim();
                self.display_notification(format!("Cell side: {}", new_side_length));
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
        self.window_dim = Point { x: width, y: height };
        self.update_dim();
        let HexDim { h, v } = self.board_dim;
        self.display_notification(format!("{}x{}", h, v));
    }
}
