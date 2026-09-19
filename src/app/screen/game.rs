use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use macroquad::camera::set_default_camera;
use macroquad::color::Color;
use macroquad::input::{mouse_position, show_mouse, KeyCode};
use macroquad::material::Material;
use macroquad::window::clear_background;
use rand::prelude::*;

use crate::app::border_hints::BorderHints;
use crate::app::distance_grid::DistanceGrid;
use crate::app::fps_control::{self, FpsControl};
use crate::app::game_context::GameContext;
use crate::app::game_mode::GameMode;
use crate::app::message;
use crate::app::message::{Message, MessageDrawable, MessageID};
use crate::app::palette::Palette;
use crate::app::prefs::{DrawGrid, HintStyle, Prefs};
use crate::app::screen::board_dim::{calculate_board_dim, calculate_offset};
use crate::app::key::Key;
use crate::app::screen::menu::{Menu, MenuEvent, Toggle};
use crate::app::screen::{Environment, Screen, Transition};
use crate::app::snake_management::{
    advance_snakes, find_collisions, handle_apple_collisions, handle_snake_collisions, relocate_covered_apples,
    spawn_snakes, update_snake_dirs,
};
use crate::app::stats::Stats;
use crate::apple::spawn::{expire_apples, food_apple, spawn_apples, spawn_bad_apples, SpawnPolicy};
use crate::basic::{CellDim, Dir, HexDim, HexPoint, Point};
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::{self, Snake};
use crate::support::flip::Flip;
use crate::support::material::snake_material;
use crate::support::mesh::{set_board_camera, Mesh};
use crate::view::snakes::OtherSnakes;
use crate::{apple, rendering};

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
    /// The part of the path over the player's eaten segments
    player_path_over_mesh: Option<Mesh>,

    /// The menu shown over the game (paused underneath), if any
    menu: Menu,
    /// Whether to leave for the main menu
    leave: bool,
    /// Where the mouse was last frame, to show the cursor when it moves
    last_mouse: (f32, f32),
}

impl Game {
    pub fn new(
        cell_dim: CellDim,
        seeds: Vec<SnakeBuilder>,
        palette: Palette,
        apple_spawn_policy: SpawnPolicy,
        mode: GameMode,
    ) -> Self {
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
                    Prefs::load(),
                    apple_spawn_policy,
                    mode,
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
            player_path_over_mesh: None,

            menu: Menu::Closed,
            leave: false,
            last_mouse: mouse_position(),
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
            self.player_path_over_mesh = None;
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
        self.player_path_over_mesh = None;

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
        relocate_covered_apples(env);
        expire_apples(env, elapsed);
        spawn_bad_apples(env, elapsed);
        let starved = env.snakes.iter().any(|snake| snake.state == snake::State::Starved);
        self.apple_mesh = None;

        if game_over || starved {
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

        let mut message = Message::default(
            format!("fps: {graphics_fps:.2}"),
            message::Position::TopLeft,
            color,
            None,
        );
        message.background = true;
        self.messages.insert(MessageID::Fps, message);
    }

    fn first_player_snake_idx(&self) -> Option<usize> {
        self.env
            .snakes
            .iter()
            .position(|snake| snake.snake_type == snake::Type::Player)
    }
}

impl Game {
    /// The options menu's toggles, a line at a time, top to bottom (special
    /// apples and the distance grid are debug keys)
    const MENU: &[&[Toggle]] = &[
        &[Toggle::DrawStyle],
        &[Toggle::Grid],
        &[Toggle::Border],
        &[Toggle::Hints],
        &[Toggle::Autopilot, Toggle::PlayerPath],
        &[Toggle::Stats],
        &[Toggle::Fps],
    ];

    /// Open the options menu, pausing the game
    fn open_menu(&mut self) {
        self.menu.open();
        if self.fps_control.state() == fps_control::State::Playing {
            self.fps_control.pause();
        }
        show_mouse(true);
    }

    fn on_menu_event(&mut self, event: MenuEvent) {
        match event {
            // the game stays paused (Space resumes it)
            MenuEvent::Closed => {}
            MenuEvent::Toggled(toggle) => self.toggled(toggle),
            MenuEvent::Restart => {
                self.restart();
                self.fps_control.play();
            }
            MenuEvent::MainMenu => self.leave = true,
        }
    }

    /// Draw the open menu over the game and act on a click
    fn draw_menu(&mut self) {
        let options: Vec<Vec<_>> = Self::MENU
            .iter()
            .map(|line| line.iter().map(|&toggle| (toggle, self.toggle_label(toggle))).collect())
            .collect();
        if let Some(event) = self.menu.draw(&options, true, &mut self.env.gtx.prefs) {
            self.on_menu_event(event);
        }
    }

    /// What a toggle's menu button says: the setting and its current value
    fn toggle_label(&self, toggle: Toggle) -> String {
        if toggle != Toggle::Autopilot {
            return toggle.label(&self.env.gtx.prefs);
        }
        let player = self.env.snakes.iter().find(|snake| snake.snake_type == snake::Type::Player);
        match player {
            _ if self.seeds.len() != 1 => "Autopilot: single player only".to_string(),
            Some(snake) if snake.autopilot.is_some() => {
                format!("Autopilot: {}", if snake.autopilot_control { "on" } else { "off" })
            }
            _ => "Autopilot: unavailable".to_string(),
        }
    }

    /// Follow up on a toggle whose preference the menu has just changed
    fn toggled(&mut self, toggle: Toggle) {
        let prefs = &self.env.gtx.prefs;

        match toggle {
            Toggle::Border => {
                self.border_mesh = None;
                self.display_notification(if prefs.draw_border { "Border on" } else { "Border off" });
            }
            Toggle::Grid => {
                let text = match prefs.draw_grid {
                    DrawGrid::Grid => "Grid",
                    DrawGrid::Dots => "Dot grid",
                    DrawGrid::None => "Grid off",
                };
                self.grid_mesh = None;
                self.display_notification(text);
            }
            Toggle::Hints => {
                let text = match prefs.hint_style {
                    HintStyle::Border => "Border hints",
                    HintStyle::Gradient => "Gradient hints",
                    HintStyle::Teleport => "Teleport hints",
                    HintStyle::None => "Hints off",
                };
                // start the new style fresh rather than fading from the old one's colors
                self.border_hints.clear();
                self.display_notification(text);
            }
            Toggle::DistanceGrid => {
                let text = if prefs.draw_distance_grid {
                    "Distance grid on"
                } else {
                    self.distance_grid_mesh = None;
                    "Distance grid off"
                };
                self.display_notification(text);
            }
            Toggle::PlayerPath => {
                let text = if prefs.draw_player_path {
                    "Path on"
                } else {
                    self.player_path_mesh = None;
                    self.player_path_over_mesh = None;
                    "Path off"
                };
                self.display_notification(text);
            }
            Toggle::Fps => {
                if !prefs.display_fps {
                    self.messages.remove(&MessageID::Fps);
                }
            }
            Toggle::Stats => {
                if !prefs.display_stats {
                    self.messages.remove(&MessageID::Stats);
                }
            }
            Toggle::Autopilot => {
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
            Toggle::DrawStyle => {
                let text = match prefs.draw_style {
                    rendering::Style::Smooth => "draw style: smooth",
                    rendering::Style::Hexagon => "draw style: hexagon",
                };
                self.snake_render = None;
                self.apple_mesh = None;
                self.display_notification(text);
            }
            Toggle::SpecialApples => {
                let text = if prefs.special_apples {
                    "Special apples enabled"
                } else {
                    // replace special apples with normal apples
                    let food = food_apple(&self.env.gtx);
                    self.env.apples.iter_mut().for_each(|apple| {
                        if matches!(apple.apple_type, apple::Type::SpawnSnake(_) | apple::Type::SpawnRain) {
                            apple.apple_type = food.clone();
                        }
                    });
                    self.apple_mesh = None;
                    "Special apples disabled"
                };
                self.display_notification(text);
            }
        }
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

        // the cursor comes back as soon as the mouse moves (macOS would
        // otherwise keep it hidden everywhere, even outside the window)
        let mouse = mouse_position();
        if mouse != self.last_mouse {
            self.last_mouse = mouse;
            show_mouse(true);
        }

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
            self.snake_render = Some(rendering::snake_mesh(
                &mut env.snakes,
                &env.apples,
                &env.gtx,
                &mut stats,
            )?);
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
        let hint_style = env.gtx.prefs.hint_style;
        let hint_mesh = Some(self.border_hints.mesh(env, player_idx, hint_style));
        // gradients go under the grid (so its lines stay untinted), recolored
        // border stretches right on top of the border
        let (gradient_hint_mesh, border_hint_mesh) = match hint_style {
            HintStyle::Gradient => (hint_mesh, None),
            HintStyle::Border | HintStyle::Teleport | HintStyle::None => (None, hint_mesh),
        };

        let (player_snake, other_snakes) = OtherSnakes::split_snakes(&mut env.snakes, player_idx);

        if env.gtx.prefs.draw_distance_grid && (self.distance_grid_mesh.is_none() || playing) {
            self.distance_grid_mesh = Some(self.distance_grid.mesh(player_snake, other_snakes, &env.gtx));
        }

        if env.gtx.prefs.draw_player_path && (self.player_path_mesh.is_none() || playing) {
            // could still be None if the player snake doesn't have an autopilot
            let meshes = rendering::player_path_mesh(player_snake, other_snakes, &env.apples, &env.gtx, &mut stats)
                .transpose()?;
            (self.player_path_mesh, self.player_path_over_mesh) = meshes.unzip();
        }

        if env.gtx.prefs.display_stats {
            stats.player_length = Some(player_snake.body.length);
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
        let before_snake = [
            &self.distance_grid_mesh,
            &gradient_hint_mesh,
            &self.grid_mesh,
            &self.player_path_mesh,
        ];
        let after_snake = [
            &self.player_path_over_mesh,
            &self.apple_mesh,
            &self.border_mesh,
            &border_hint_mesh,
            &self.portal_mesh,
        ];

        let has_snake = self.snake_render.as_ref().is_some_and(|r| !r.shaded.is_empty());
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

        if self.menu.is_open() {
            set_default_camera();
            self.draw_menu();
        }

        Ok(())
    }

    fn key_down_event(&mut self, key: Key) -> Result<()> {
        use KeyCode::*;

        let (used, event) = self.menu.key_pressed(key, &mut self.env.gtx.prefs);
        if let Some(event) = event {
            self.on_menu_event(event);
        }
        // Playing with the keyboard hides the cursor until the mouse moves.
        // (Esc that closed the menu counts; Esc that is about to open it
        // doesn't.)
        if self.env.gtx.prefs.hide_cursor && !self.menu.is_open() && (used || key != Key::Code(Escape)) {
            show_mouse(false);
        }
        if used {
            return Ok(());
        }
        if key == Key::Code(Escape) {
            self.open_menu();
            return Ok(());
        }

        // keys as the layout names them (see `app::key`)
        match key {
            // play/pause, or start over after a game over; ignored while the
            // menu is open
            Key::Code(Space) if !self.menu.is_open() => match self.fps_control.state() {
                fps_control::State::GameOver => {
                    self.restart();
                    self.fps_control.play();
                }
                fps_control::State::Playing => self.fps_control.pause(),
                fps_control::State::Paused => self.fps_control.play(),
            },
            // debug toggles
            Key::Char(c @ ('X' | 'D')) => {
                let toggle = if c == 'X' { Toggle::SpecialApples } else { Toggle::DistanceGrid };
                toggle.apply(&mut self.env.gtx.prefs);
                self.toggled(toggle);
            }
            Key::Char(c @ '1'..='9') => {
                let new_food = c.to_digit(10).expect("a digit") as f32;
                self.env.gtx.prefs.apple_food = new_food;
                // change existing apples
                for apple in &mut self.env.apples {
                    if let apple::Type::Eat(food) = &mut apple.apple_type {
                        *food = new_food;
                    }
                }
                self.display_notification(format!("Apple food: {new_food}"));
            }
            Key::Code(k @ (Down | Up)) => {
                let factor = if k == Down { 0.9 } else { 1. / 0.9 };
                let mut new_side_length = self.env.gtx.cell_dim.side * factor;
                new_side_length = new_side_length.clamp(Self::CELL_SIDE_MIN, Self::CELL_SIDE_MAX);
                self.env.gtx.cell_dim = CellDim::from(new_side_length);
                self.update_dim();
                self.display_notification(format!("Cell side: {new_side_length}"));
            }
            Key::Char(c @ ('[' | ']')) => {
                let speed = if c == '[' {
                    self.fps_control.slower()
                } else {
                    self.fps_control.faster()
                };
                self.display_notification(format!("Speed: {speed}x"));
            }
            _ => {
                if self.fps_control.state() == fps_control::State::Playing {
                    for snake in &mut self.env.snakes {
                        snake.controller.key_pressed(key, &self.env.gtx)
                    }
                }
            }
        }

        Ok(())
    }

    fn transition(&mut self) -> Option<Transition> {
        self.leave.then_some(Transition::Pop)
    }

    // TODO: forbid resizing in-game
    fn resize_event(&mut self, _width: f32, _height: f32) -> Result<()> {
        self.update_dim();
        let HexDim { h, v } = self.env.gtx.board_dim;
        self.display_notification(format!("{h}x{v}"));
        Ok(())
    }
}
