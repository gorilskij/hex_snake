use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use enum_rotate::EnumRotate;
use macroquad::camera::set_default_camera;
use macroquad::color::Color;
use macroquad::input::{show_mouse, KeyCode};
use macroquad::material::Material;
use macroquad::window::clear_background;
use rand::prelude::*;

use crate::app::border_hints::BorderHints;
use crate::app::distance_grid::DistanceGrid;
use crate::app::fps_control::{self, FpsControl};
use crate::app::game_context::GameContext;
use crate::app::message;
use crate::app::message::{Message, MessageDrawable, MessageID};
use crate::app::palette::Palette;
use crate::app::prefs::{DrawGrid, Prefs};
use crate::app::screen::board_dim::{calculate_board_dim, calculate_offset};
use crate::app::screen::{Environment, Screen};
use crate::app::snake_management::{
    advance_snakes, find_collisions, handle_apple_collisions, handle_snake_collisions, spawn_snakes, update_snake_dirs,
};
use crate::app::stats::Stats;
use crate::apple::spawn::{spawn_apples, SpawnPolicy};
use crate::apple::{self, Apple};
use crate::basic::{CellDim, Dir, HexDim, HexPoint, Point};
use crate::rendering;
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::{self, Snake};
use crate::support::flip::Flip;
use crate::support::material::snake_material;
use crate::support::mesh::{set_board_camera, Mesh};
use crate::view::snakes::OtherSnakes;

pub struct Game {
    env: Environment,
    fps_control: FpsControl,

    /// Offset to center the grid in the window
    offset: Point,

    seeds: Vec<SnakeBuilder>,
    animated_apples: bool,

    distance_grid: DistanceGrid,
    border_hints: BorderHints,

    messages: HashMap<MessageID, Message>,

    grid_mesh: Option<Mesh>,
    border_mesh: Option<Mesh>,
    portal_mesh: Option<Mesh>,
    snake_render: Option<rendering::SnakeRender>,
    /// Shader material for coloring snakes from their palette LUT. Compiled
    /// lazily on first draw (needs a live GL context).
    snake_material: Option<Material>,
    apple_mesh: Option<Mesh>,
    distance_grid_mesh: Option<Mesh>,
    player_path_mesh: Option<Mesh>,
}

impl Game {
    pub fn new(cell_dim: CellDim, seeds: Vec<SnakeBuilder>, palette: Palette, apple_spawn_policy: SpawnPolicy) -> Self {
        assert!(!seeds.is_empty(), "No players specified");

        let mut this = Self {
            env: Environment {
                snakes: vec![],
                apples: vec![],
                // no portals in the minimal wasm build
                portals: vec![],
                gtx: GameContext::new(
                    // updated immediately after creation
                    HexPoint { h: 0, v: 0 },
                    cell_dim,
                    palette,
                    Prefs::default(),
                    apple_spawn_policy,
                ),
                rng: thread_rng(),
            },
            fps_control: FpsControl::new(),

            // updated immediately after creation
            offset: Point { x: 0., y: 0. },

            seeds,
            animated_apples: false,

            distance_grid: DistanceGrid::new(),
            border_hints: BorderHints::new(),

            messages: HashMap::new(),

            grid_mesh: None,
            border_mesh: None,
            portal_mesh: None,
            snake_render: None,
            snake_material: None,
            apple_mesh: None,
            distance_grid_mesh: None,
            player_path_mesh: None,
        };
        this.update_dim();
        // warning: this spawns apples before there are any snakes
        this.restart();
        this
    }

    fn update_dim(&mut self) {
        let env = &mut self.env;

        let board_dim = calculate_board_dim(env.gtx.cell_dim);

        self.offset = calculate_offset(board_dim, env.gtx.cell_dim);

        if env.gtx.board_dim != board_dim {
            env.gtx.board_dim = board_dim;

            // restart if player snake head has left board limits
            if env
                .snakes
                .iter()
                .any(|s| s.snake_type == snake::Type::Player && !board_dim.contains(s.head().pos))
            {
                println!("warning: player snake outside of board, restarting");
                self.restart();
            } else {
                // remove snakes outside of board limits
                env.snakes.retain(move |snake| board_dim.contains(snake.head().pos));

                // remove apples outside of board limits
                env.apples.retain(move |apple| board_dim.contains(apple.pos));
                self.spawn_apples();
            }

            // update portals
            self.env
                .portals
                .iter_mut()
                .for_each(move |portal| portal.update(board_dim));

            // invalidate
            self.grid_mesh = None;
            self.border_mesh = None;
            self.portal_mesh = None;
            self.apple_mesh = None;
            self.snake_render = None;
            self.distance_grid_mesh = None;
            self.distance_grid.invalidate();
            self.border_hints.clear();
            self.player_path_mesh = None;
        }
    }

    // TODO: R as a restart shortcut but only in debug mode
    fn restart(&mut self) {
        let env = &mut self.env;

        env.snakes.clear();
        env.apples.clear();

        self.snake_render = None;
        self.apple_mesh = None;
        self.distance_grid_mesh = None;
        self.player_path_mesh = None;

        // seeds without a defined spawn point
        let unpositioned = self
            .seeds
            .iter()
            .filter(|seed| !matches!(seed.snake_type, Some(snake::Type::Simulated)))
            .count();

        // TODO: clean this mess
        let mut unpositioned_dir = Dir::U;
        let mut unpositioned_h_pos: Box<dyn Iterator<Item = isize>> = if unpositioned > 0 {
            const DISTANCE_BETWEEN_SNAKES: isize = 1;

            let total_width = (unpositioned - 1) as isize * DISTANCE_BETWEEN_SNAKES + 1;
            assert!(total_width < env.gtx.board_dim.h, "snakes spread too wide");

            let half = total_width / 2;
            let middle = env.gtx.board_dim.h / 2;
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
                    env.snakes.push(seed.build().unwrap());
                }
                _ => {
                    env.snakes.push(
                        seed.clone()
                            .pos(HexPoint {
                                h: unpositioned_h_pos.next().unwrap(),
                                v: env.gtx.board_dim.v / 2,
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

    /// Advance the world by one tick, which must be short enough that no snake
    /// crosses more than one cell boundary (see [`Screen::update`]).
    /// Return value indicates whether any snake has advanced through a cell boundary
    fn advance_snakes(&mut self, elapsed: Duration) -> Result<bool> {
        let env = &mut self.env;

        let new_cells_occupied = advance_snakes(env, elapsed);

        // if only ephemeral AIs are left, kill all other snakes
        let dying_or_ephemeral = |snake: &Snake| {
            matches!(snake.state, snake::State::Dying)
                || matches!(
                    snake.snake_type,
                    snake::Type::Competitor { life: Some(_) } | snake::Type::Killer { life: Some(_) }
                )
        };
        if env.snakes.iter().all(dying_or_ephemeral) {
            for snake in &mut env.snakes {
                snake.die();
            }
        }

        if env.snakes.is_empty() {
            self.fps_control.game_over();
            return Ok(false);
        }

        let collisions = find_collisions(env);
        let game_over = handle_snake_collisions(env, &collisions);
        let seeds = handle_apple_collisions(env, &collisions);
        self.apple_mesh = None;

        if game_over {
            self.fps_control.game_over()
        }

        spawn_snakes(&mut self.env, seeds).context("Game::advance_snakes")?;

        Ok(new_cells_occupied)
    }
}

impl Game {
    /// Bounds for the length of one of the six sides of a cell
    const CELL_SIDE_MIN: f32 = 5.;
    const CELL_SIDE_MAX: f32 = 1000.;

    fn spawn_apples(&mut self) {
        spawn_apples(&mut self.env);
        self.apple_mesh = None;
    }

    fn get_message_drawables(&mut self) -> Vec<MessageDrawable> {
        // draw messages and remove the ones that have
        // outlived their durations
        let mut drawables = vec![];
        let mut remove = vec![];

        self.messages
            .iter()
            .for_each(|(id, message)| match message.get_drawable() {
                Some(drawable) => drawables.push(drawable),
                None => remove.push(*id),
            });

        remove.iter().for_each(|id| {
            self.messages.remove(id);
        });

        drawables
    }

    /// Display a notification message in the top-right
    /// corner with limited duration and default parameters,
    /// overwrite any previous notification message
    fn display_notification<S: ToString>(&mut self, text: S) {
        self.messages.insert(
            MessageID::Notification,
            Message::default(
                text.to_string(),
                message::Position::TopRight,
                crate::color::WHITE,
                Some(self.env.gtx.prefs.message_duration),
            ),
        );
    }

    /// Show game and graphics FPS information in the
    /// top-left corner
    fn update_fps_message(&mut self) {
        let graphics_fps = self.fps_control.measured_graphics_fps();

        let graphics_fps_undershoot = (60. - graphics_fps) / graphics_fps;
        let color = if graphics_fps_undershoot > 0.05 {
            // > 5% undershoot: red
            Color::from_rgba(200, 0, 0, 255)
        } else if graphics_fps_undershoot > 0.02 {
            // > 2% undershoot: orange
            Color::from_rgba(235, 168, 52, 255)
        } else {
            // at or overshoot: white
            crate::color::WHITE
        };

        self.messages.insert(
            MessageID::Fps,
            Message::default(
                format!("fps: {graphics_fps:.2}"),
                message::Position::TopLeft,
                color,
                None,
            ),
        );
    }

    fn first_player_snake_idx(&self) -> Option<usize> {
        self.env
            .snakes
            .iter()
            .position(|snake| snake.snake_type == snake::Type::Player)
    }
}

impl Screen for Game {
    fn update(&mut self) -> Result<()> {
        /// Most cells the fastest snake may travel in one tick, so no snake
        /// crosses more than one cell boundary per tick.
        const MAX_TICK_CELLS: f32 = 0.5;

        if let Some(elapsed) = self.fps_control.update() {
            // at high speeds a frame covers many cells: split it into ticks
            let fastest = self.env.snakes.iter().map(|snake| snake.speed).fold(0., f32::max);
            let ticks = (elapsed.as_secs_f32() * fastest / MAX_TICK_CELLS).ceil().max(1.) as u32;
            let tick = elapsed / ticks;

            for _ in 0..ticks {
                if self.fps_control.state() != fps_control::State::Playing {
                    break;
                }

                if self.advance_snakes(tick).context("Game::update")? {
                    self.spawn_apples();
                }

                // Poll controllers only once the tick's world state is final
                // (collisions handled, eaten apples removed, new apples spawned):
                // a decision is locked in for the rest of the cell, so deciding
                // against a stale world made the autopilot overshoot apples.
                update_snake_dirs(&mut self.env);
            }
        }

        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        self.fps_control.graphics_frame();

        if self.env.gtx.prefs.display_fps {
            self.update_fps_message();
        }

        let env = &mut self.env;
        let mut stats = Stats::default();
        let playing = self.fps_control.state() == fps_control::State::Playing;

        if self.grid_mesh.is_none() {
            match env.gtx.prefs.draw_grid {
                DrawGrid::Grid => self.grid_mesh = Some(rendering::grid_mesh(&env.gtx)?),
                DrawGrid::Dots => self.grid_mesh = Some(rendering::grid_dot_mesh(&env.gtx)?),
                _ => {}
            }
        }

        if env.gtx.prefs.draw_border && self.border_mesh.is_none() {
            self.border_mesh = Some(rendering::border_mesh(&env.gtx)?);
        }

        if self.portal_mesh.is_none() {
            self.portal_mesh = Some(rendering::portal_mesh(&mut env.portals, &env.gtx, &mut stats)?);
        }

        if self.snake_render.is_none() || playing {
            self.snake_render = Some(rendering::snake_mesh(&mut env.snakes, &env.apples, &env.gtx, &mut stats)?);
        }

        if env.apples.is_empty() {
            self.apple_mesh = None;
        } else if self.apple_mesh.is_none() || self.animated_apples {
            // only recompute apple mesh if there are animated apples
            self.apple_mesh = Some(rendering::apple_mesh(
                &env.apples,
                &env.gtx,
                self.fps_control.elapsed_total(),
                &mut stats,
            )?);
        }

        let player_idx = self.first_player_snake_idx().expect("no player snake");
        let env = &mut self.env;

        // rebuilt every frame: hints fade in real time, even while paused
        let border_hint_mesh = Some(self.border_hints.mesh(env, player_idx));

        let (player_snake, other_snakes) = OtherSnakes::split_snakes(&mut env.snakes, player_idx);

        if env.gtx.prefs.draw_distance_grid && (self.distance_grid_mesh.is_none() || playing) {
            self.distance_grid_mesh = Some(self.distance_grid.mesh(player_snake, other_snakes, &env.gtx));
        }

        if env.gtx.prefs.draw_player_path && (self.player_path_mesh.is_none() || playing) {
            // could still be None if the player snake doesn't have an autopilot
            self.player_path_mesh =
                rendering::player_path_mesh(player_snake, other_snakes, &env.apples, &env.gtx, &mut stats)
                    .transpose()?;
        }

        if env.gtx.prefs.display_stats {
            let message = stats.get_stats_message();
            self.messages.insert(MessageID::Stats, message);
        }

        // Compile the snake shader lazily (GL context is live during draw).
        if self.snake_material.is_none() {
            self.snake_material = Some(snake_material()?);
        }

        let message_drawables = self.get_message_drawables();

        // Meshes drawn on the default material, split around the snake so the
        // snake keeps its old z-order (below apples/border, above grid/paths).
        // Border hints go under the grid (and border), so its lines stay untinted.
        let before_snake = [
            &self.distance_grid_mesh,
            &border_hint_mesh,
            &self.grid_mesh,
            &self.player_path_mesh,
        ];
        let after_snake = [&self.apple_mesh, &self.border_mesh, &self.portal_mesh];

        let has_snake = self
            .snake_render
            .as_ref()
            .is_some_and(|r| !r.shaded.is_empty());
        let has_plain = before_snake.iter().chain(after_snake.iter()).any(|m| m.is_some());

        if !message_drawables.is_empty() || has_snake || has_plain {
            clear_background(self.env.gtx.palette.background_color);

            // Every board mesh renders under the same board camera, so set it
            // once here rather than per-mesh: each set_camera flushes the GPU
            // batch, so per-mesh camera sets were the bulk of the frame's cost.
            set_board_camera(self.offset);

            for mesh in before_snake.into_iter().flatten() {
                mesh.draw();
            }

            if let Some(render) = &self.snake_render {
                let material = self.snake_material.as_ref().unwrap();
                for (mesh, _lut) in &render.shaded {
                    mesh.draw_shaded(material);
                }
            }

            for mesh in after_snake.into_iter().flatten() {
                mesh.draw();
            }

            // Text lives in screen space; switch to the default camera once for
            // all messages rather than once per message.
            if !message_drawables.is_empty() {
                set_default_camera();
                for drawable in message_drawables {
                    drawable.draw();
                }
            }
        }

        Ok(())
    }

    fn key_down_event(&mut self, keycode: KeyCode) -> Result<()> {
        let prefs = &mut self.env.gtx.prefs;

        if prefs.hide_cursor {
            show_mouse(false);
        }

        use KeyCode::*;

        let numeric_keys = [Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9];

        // TODO: also tie these to a keymap (dvorak-centric for now)
        match keycode {
            Escape => match self.fps_control.state() {
                fps_control::State::GameOver => {
                    self.restart();
                    self.fps_control.play();
                }
                fps_control::State::Playing => {
                    self.fps_control.pause();
                }
                fps_control::State::Paused => self.fps_control.play(),
            },
            B => {
                let text = match prefs.draw_border.flip() {
                    true => "Border on",
                    false => "Border off",
                };
                self.border_mesh = None;
                self.display_notification(text);
            }
            G => {
                let text = match prefs.draw_grid.rotate_next() {
                    DrawGrid::Grid => "Grid",
                    DrawGrid::Dots => "Dot grid",
                    DrawGrid::None => "Grid off",
                };
                self.grid_mesh = None;
                self.display_notification(text);
            }
            D => {
                let text = if prefs.draw_distance_grid.flip() {
                    "Distance grid on"
                } else {
                    self.distance_grid_mesh = None;
                    "Distance grid off"
                };
                self.display_notification(text);
            }
            P => {
                let text = if prefs.draw_player_path.flip() {
                    "Path on"
                } else {
                    self.player_path_mesh = None;
                    "Path off"
                };
                self.display_notification(text);
            }
            F => {
                if !prefs.display_fps.flip() {
                    self.messages.remove(&MessageID::Fps);
                }
            }
            S => {
                if !prefs.display_stats.flip() {
                    self.messages.remove(&MessageID::Stats);
                }
            }
            A => {
                // only apply if there is exactly one player snake
                if self.seeds.len() == 1 {
                    let player_snake = self
                        .env
                        .snakes
                        .iter_mut()
                        .find(|snake| snake.snake_type == snake::Type::Player)
                        .unwrap();

                    if player_snake.autopilot.is_some() {
                        let text = if player_snake.autopilot_control.flip() {
                            "Autopilot on"
                        } else {
                            player_snake.controller.reset(player_snake.body.dir);
                            "Autopilot off"
                        };
                        self.display_notification(text);
                    } else {
                        self.display_notification("Autopilot not available");
                    }
                } else {
                    self.display_notification(format!("Can't use autopilot with {} players", self.seeds.len()));
                }
            }
            Tab => {
                let text;
                match prefs.draw_style {
                    rendering::Style::Hexagon => {
                        prefs.draw_style = rendering::Style::Smooth;
                        text = "draw style: smooth";
                    }
                    rendering::Style::Smooth => {
                        prefs.draw_style = rendering::Style::Hexagon;
                        text = "draw style: hexagon";
                    }
                }
                self.snake_render = None;
                self.apple_mesh = None;
                self.display_notification(text);
            }
            X => {
                let text = if prefs.special_apples.flip() {
                    "Special apples enabled"
                } else {
                    // replace special apples with normal apples
                    let apple_food = prefs.apple_food;
                    self.env.apples.iter_mut().for_each(|apple| {
                        if !matches!(apple.apple_type, apple::Type::Eat(_)) {
                            *apple = Apple {
                                pos: apple.pos,
                                apple_type: apple::Type::Eat(apple_food),
                            }
                        }
                    });
                    self.apple_mesh = None;
                    "Special apples disabled"
                };
                self.display_notification(text);
            }
            k if let Some(idx) = numeric_keys.iter().position(|nk| *nk == k) => {
                let new_food = idx as f32 + 1.0;
                prefs.apple_food = new_food;
                // change existing apples
                for apple in &mut self.env.apples {
                    if let apple::Type::Eat(food) = &mut apple.apple_type {
                        *food = new_food;
                    }
                }
                self.display_notification(format!("Apple food: {new_food}"));
            }
            k @ Down | k @ Up => {
                let factor = if k == Down { 0.9 } else { 1. / 0.9 };
                let mut new_side_length = self.env.gtx.cell_dim.side * factor;
                new_side_length = new_side_length.clamp(Self::CELL_SIDE_MIN, Self::CELL_SIDE_MAX);
                self.env.gtx.cell_dim = CellDim::from(new_side_length);
                self.update_dim();
                self.display_notification(format!("Cell side: {new_side_length}"));
            }
            k @ LeftBracket | k @ RightBracket => {
                let speed = if k == LeftBracket {
                    self.fps_control.slower()
                } else {
                    self.fps_control.faster()
                };
                self.display_notification(format!("Speed: {speed}x"));
            }
            k => {
                if self.fps_control.state() == fps_control::State::Playing {
                    for snake in &mut self.env.snakes {
                        snake.controller.key_pressed(k)
                    }
                }
            }
        }

        Ok(())
    }

    // TODO: forbid resizing in-game
    fn resize_event(&mut self, _width: f32, _height: f32) -> Result<()> {
        self.update_dim();
        let HexDim { h, v } = self.env.gtx.board_dim;
        self.display_notification(format!("{h}x{v}"));
        Ok(())
    }
}
