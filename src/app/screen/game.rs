use std::collections::HashMap;

use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Color, DrawParam, Mesh};
use ggez::Context;
use rand::prelude::*;

use crate::app::distance_grid::DistanceGrid;
use crate::app::fps_control::{self, FpsControl};
use crate::app::game_context::GameContext;
use crate::app::message::{Message, MessageDrawable, MessageID};
use crate::app::palette::Palette;
use crate::app::prefs::DrawGrid;
use crate::app::screen::board_dim::{calculate_board_dim, calculate_offset};
use crate::app::screen::Environment;
use crate::app::snake_management::{
    advance_snakes, find_collisions, handle_collisions, spawn_snakes,
};
use crate::app::stats::Stats;
use crate::apple::spawn::{spawn_apples, SpawnPolicy};
use crate::apple::{self, Apple};
use crate::basic::{CellDim, Dir, Food, HexDim, HexPoint, Point};
use crate::error::{Error, ErrorConversion, Result};
use crate::rendering;
use crate::snake::{self, Snake};
use crate::support::flip::Flip;
use crate::support::invert::Invert;
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
    animated_apples: bool,

    rng: ThreadRng,

    distance_grid: DistanceGrid,

    messages: HashMap<MessageID, Message>,

    grid_mesh: Option<Mesh>,
    border_mesh: Option<Mesh>,
    snake_mesh: Option<Mesh>,
    apple_mesh: Option<Mesh>,
    distance_grid_mesh: Option<Mesh>,
    player_path_mesh: Option<Mesh>,

    // TODO: move this mechanism to Control
    /// Consider the draw cache invalid for the
    /// next n frames, forces a redraw even if
    /// nothing changed, this is necessary to
    /// avoid visual glitches
    draw_cache_invalid: usize,
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
            animated_apples: false,

            rng: thread_rng(),

            distance_grid: DistanceGrid::new(),

            messages: HashMap::new(),

            grid_mesh: None,
            border_mesh: None,
            snake_mesh: None,
            apple_mesh: None,
            distance_grid_mesh: None,
            player_path_mesh: None,

            draw_cache_invalid: 0,
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
            self.apple_mesh = None;
            self.snake_mesh = None;
            self.distance_grid_mesh = None;
            self.distance_grid.invalidate();
            self.player_path_mesh = None;
        }
    }

    fn restart(&mut self) {
        self.snakes.clear();
        self.apples.clear();

        self.snake_mesh = None;
        self.apple_mesh = None;
        self.distance_grid_mesh = None;
        self.player_path_mesh = None;

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

    fn advance_snakes(&mut self, ctx: &Context) -> Result {
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
        self.animated_apples = self.animated_apples
            || new_apples
                .iter()
                .any(|apple| apple.apple_type.is_animated());
        self.apples.extend(new_apples.into_iter());
    }

    fn get_message_drawables(&mut self, ctx: &Context) -> Vec<MessageDrawable> {
        // draw messages and remove the ones that have
        // outlived their durations
        let mut drawables = vec![];
        let mut remove = vec![];

        self.messages
            .iter()
            .for_each(|(id, message)| match message.get_drawable(ctx) {
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

        let game_fps_undershoot = (self.fps_control.fps() - game_fps) / game_fps;
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
                format!("u: {game_fps:.2} g: {graphics_fps:.2}"),
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
    fn update(&mut self, ctx: &mut Context) -> Result {
        while self.fps_control.can_update(&mut self.gtx) {
            self.advance_snakes(ctx).with_trace_step("Game::update")?;
            self.spawn_apples();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result {
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

        let mut stats = Stats::default();

        let playing = self.fps_control.state() == fps_control::State::Playing;

        if playing {
            // Update the direction of the snake early
            // to see it turning as soon as possible,
            // this could happen in the middle of a
            // game frame. Repeated update s during the
            // same game frame are blocked
            for idx in 0..self.snakes.len() {
                let (snake, other_snakes) = OtherSnakes::split_snakes(&mut self.snakes, idx);
                snake.update_dir(other_snakes, &self.apples, &self.gtx, ctx);
            }
        }

        if self.grid_mesh.is_none() {
            match self.gtx.prefs.draw_grid {
                DrawGrid::Grid => self.grid_mesh = Some(rendering::grid_mesh(&self.gtx, ctx)?),
                DrawGrid::Dots => self.grid_mesh = Some(rendering::grid_dot_mesh(&self.gtx, ctx)?),
                _ => {}
            }
        }

        if self.gtx.prefs.draw_border && self.border_mesh.is_none() {
            self.border_mesh = Some(rendering::border_mesh(&self.gtx, ctx)?);
        }

        if self.snake_mesh.is_none() || playing {
            self.snake_mesh = Some(rendering::snake_mesh(
                &mut self.snakes,
                &self.gtx,
                ctx,
                &mut stats,
            )?);
        }

        // only recompute apple mesh if there are animated apples
        if self.apple_mesh.is_none() || self.animated_apples {
            self.apple_mesh = Some(rendering::apple_mesh(
                &self.apples,
                &self.gtx,
                ctx,
                &mut stats,
            )?);
        }

        let player_idx = self.first_player_snake_idx().expect("no player snake");
        let (player_snake, other_snakes) = OtherSnakes::split_snakes(&mut self.snakes, player_idx);

        if self.gtx.prefs.draw_distance_grid && (self.distance_grid_mesh.is_none() || playing) {
            self.distance_grid_mesh =
                Some(
                    self.distance_grid
                        .mesh(player_snake, other_snakes, ctx, &self.gtx)?,
                );
        }

        if self.gtx.prefs.draw_player_path && (self.player_path_mesh.is_none() || playing) {
            // could still be None if the player snake doesn't have an autopilot
            self.player_path_mesh = rendering::player_path_mesh(
                player_snake,
                other_snakes,
                &self.apples,
                ctx,
                &self.gtx,
                &mut stats,
            )
            .invert()?;
        }

        if self.gtx.prefs.display_stats {
            let message = stats.get_stats_message();
            self.messages.insert(MessageID::Stats, message);
        }

        let message_drawables = self.get_message_drawables(ctx);

        let meshes = [
            &self.distance_grid_mesh,
            &self.grid_mesh,
            &self.player_path_mesh,
            &self.snake_mesh,
            &self.apple_mesh,
            &self.border_mesh,
        ];

        if !message_drawables.is_empty() || meshes.iter().any(|mesh| mesh.is_some()) {
            graphics::clear(ctx, self.gtx.palette.background_color);

            let draw_param = DrawParam::default().dest(self.offset);
            for mesh in meshes.into_iter().flatten() {
                graphics::draw(ctx, mesh, draw_param)?;
            }

            for drawable in message_drawables {
                drawable.draw(ctx)?;
            }

            graphics::present(ctx)
                .map_err(Error::from)
                .with_trace_step("Game::draw")?;
        }

        Ok(())
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
                let text = match self.gtx.prefs.draw_grid.rotate() {
                    DrawGrid::Grid => {
                        self.gtx.prefs.draw_border = true;
                        "Grid"
                    }
                    DrawGrid::Dots => {
                        self.gtx.prefs.draw_border = true;
                        "Dots"
                    }
                    DrawGrid::None => {
                        self.gtx.prefs.draw_border = false;
                        "Grid off"
                    }
                };
                self.grid_mesh = None;
                self.border_mesh = None;
                self.display_notification(text);
            }
            D => {
                let text = if self.gtx.prefs.draw_distance_grid.flip() {
                    "Distance grid on"
                } else {
                    self.distance_grid_mesh = None;
                    "Distance grid off"
                };
                self.display_notification(text);
            }
            P => {
                let text = if self.gtx.prefs.draw_player_path.flip() {
                    "Path on"
                } else {
                    self.player_path_mesh = None;
                    "Path off"
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
            A => {
                // only apply if there is exactly one player snake
                if self.seeds.len() == 1 {
                    let player_snake = self
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
                    self.display_notification(format!(
                        "Can't use autopilot with {} players",
                        self.seeds.len()
                    ));
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
                self.display_notification(format!("fps: {new_fps}"));
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
                self.display_notification(format!("fps: {new_fps}"));
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
                self.snake_mesh = None;
                self.apple_mesh = None;
                self.display_notification(text);
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
                    self.apple_mesh = None;
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
                self.display_notification(format!("Apple food: {new_food}"));
            }
            k @ Down | k @ Up => {
                let factor = if k == Down { 0.9 } else { 1. / 0.9 };
                let mut new_side_length = self.gtx.cell_dim.side * factor;
                new_side_length = new_side_length.clamp(Self::CELL_SIDE_MIN, Self::CELL_SIDE_MAX);
                self.gtx.cell_dim = CellDim::from(new_side_length);
                self.update_dim(ctx);
                self.display_notification(format!("Cell side: {new_side_length}"));
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
        self.display_notification(format!("{h}x{v}"));
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

    fn add_snake(&mut self, snake_builder: &snake::Builder) -> Result {
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

    // TODO: notification system so autopilots can adapt
    fn remove_apple(&mut self, index: usize) -> Apple {
        let apple = self.apples.remove(index);
        self.animated_apples = self
            .apples
            .iter()
            .any(|apple| apple.apple_type.is_animated());
        self.apple_mesh = None;
        apple
    }

    fn gtx(&self) -> &GameContext {
        &self.gtx
    }

    fn rng(&mut self) -> &mut ThreadRng {
        &mut self.rng
    }
}
