use std::f32::consts::TAU;
use std::time::Duration;

use anyhow::Result;
use enum_map_lite::enum_map;
use macroquad::camera::set_default_camera;
use macroquad::color::Color;
use macroquad::input::{show_mouse, KeyCode};
use macroquad::material::Material;
use macroquad::time::get_frame_time;
use macroquad::window::{clear_background, screen_height, screen_width};
use rand::prelude::*;

use super::Game;
use crate::app::game_context::GameContext;
use crate::app::game_mode::GameMode;
use crate::app::key::KeyPress;
use crate::app::prefs::Prefs;
use crate::app::screen::menu::{Menu, Toggle};
use crate::app::screen::{Environment, Screen, Transition};
use crate::app::snake_management::{advance_snakes, update_snake_dirs};
use crate::app::stats::Stats;
use crate::app::{self};
use crate::apple::spawn::SpawnPolicy;
use crate::basic::{CellDim, Dir, HexDim, HexPoint, Point};
use crate::button::style::{BUTTON_CELL_DIM, BUTTON_COLOR, FONT_SIZE, STROKE_THICKNESS};
use crate::button::{Button, ButtonData};
use crate::rendering;
use crate::rendering::shape::{Hexagon, Shape, ShapePoints, TriangleArrowLeft, WideHexagon};
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::eat_mechanics::{EatBehavior, EatMechanics};
use crate::snake::{self, PaletteTemplate, Snake};
use crate::snake_control::Template;
use crate::support::material::snake_material;
use crate::support::text::{draw_text, measure_text};
use crate::support::mesh::set_board_camera;

/// Palettes a player can pick from
fn palettes() -> Vec<PaletteTemplate> {
    vec![
        PaletteTemplate::rainbow(),
        PaletteTemplate::pastel_rainbow(),
        PaletteTemplate::dark_rainbow(),
        PaletteTemplate::green_to_red(),
        PaletteTemplate::dark_blue_to_red(),
        PaletteTemplate::alternating_white(),
        PaletteTemplate::solid_white_red(),
        PaletteTemplate::gray_gradient(1.),
    ]
}

/// A small board with a snake circling on it, previewing a player's palette,
/// with arrow buttons below to flip through the palettes.
struct SnakeDemo {
    /// Top-left of the board, in screen coordinates
    pos: Point,
    env: Environment,

    palettes: Vec<PaletteTemplate>,
    current_palette: usize,
    left_button: Button,
    right_button: Button,
}

impl SnakeDemo {
    const BOARD_DIM: HexDim = HexPoint { h: 11, v: 8 };

    fn new(cell_dim: CellDim, pos: Point, app_palette: app::Palette) -> Self {
        let palettes = palettes();

        // side 2 makes a closed loop of 6 sides of 3 cells, longer than the
        // snake, so it never runs into itself
        let snake = SnakeBuilder::default()
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::new(
                enum_map! { _ => EatBehavior::Crash },
                enum_map! { _ => enum_map! { _ => EatBehavior::Crash } },
            ))
            .controller(Template::demo_hexagon_pattern(Dir::U, 2))
            .pos(HexPoint { h: 2, v: 5 })
            .dir(Dir::U)
            .len(10)
            .speed(4.)
            .palette(palettes[0])
            .build()
            .expect("the demo snake is fully specified");

        let (left, right) = Self::arrow_buttons(1.);

        let mut this = Self {
            pos,
            env: Environment {
                snakes: vec![snake],
                apples: vec![],
                portals: vec![],
                gtx: GameContext::new(
                    Self::BOARD_DIM,
                    cell_dim,
                    app_palette,
                    Prefs::default(),
                    SpawnPolicy::None,
                    GameMode::Classic,
                ),
                rng: thread_rng(),
            },
            palettes,
            current_palette: 0,
            // placed and sized by `layout`
            left_button: Button::click(Point::zero(), left),
            right_button: Button::click(Point::zero(), right),
        };
        this.layout(cell_dim, pos);
        this
    }

    /// The previous/next palette buttons, at `scale` times their full size
    fn arrow_buttons(scale: f32) -> (ButtonData, ButtonData) {
        let cell_dim = BUTTON_CELL_DIM * scale;
        let stroke_thickness = STROKE_THICKNESS * scale;

        let outer_shape = Hexagon::new(cell_dim);
        let inner_shape = TriangleArrowLeft::new(cell_dim * 0.6);
        let arrow_offset = outer_shape.center() - inner_shape.center();
        let left = ButtonData::new(outer_shape.clone(), stroke_thickness, BUTTON_COLOR).inner_shape(
            inner_shape.clone(),
            arrow_offset,
            stroke_thickness,
            BUTTON_COLOR,
        );
        let right = ButtonData::new(outer_shape, stroke_thickness, BUTTON_COLOR).inner_shape(
            inner_shape.rotate_clockwise_about_center(TAU / 2.),
            arrow_offset,
            stroke_thickness,
            BUTTON_COLOR,
        );
        (left, right)
    }

    /// Size of the demo board (without the buttons) for a given cell size
    fn board_size(cell_dim: CellDim) -> Point {
        let CellDim { side, cos, .. } = cell_dim;
        Point {
            x: Self::BOARD_DIM.h as f32 * (side + cos) + cos,
            y: (Self::BOARD_DIM.v as f32 + 0.5) * cell_dim.height(),
        }
    }

    /// Move the demo to `pos` and resize it to `cell_dim`, placing the arrow
    /// buttons below the board
    fn layout(&mut self, cell_dim: CellDim, pos: Point) {
        self.pos = pos;
        self.env.gtx.cell_dim = cell_dim;

        let board = Self::board_size(cell_dim);

        // The buttons are centered at a quarter and three quarters of the
        // board's width, half a board apart. Once the board is too narrow for
        // two full-size buttons (outline stroke included) and a gap in that
        // space, they shrink so they keep the gap.
        const GAP: f32 = 10.;
        let full_width = BUTTON_CELL_DIM.width() + STROKE_THICKNESS;
        let scale = ((board.x / 2. - GAP) / full_width).clamp(0., 1.);
        let (left, right) = Self::arrow_buttons(scale);
        let button_dim = BUTTON_CELL_DIM * scale;

        let y = pos.y + board.y + button_dim.side;
        let x = |fraction: f32| pos.x + fraction * board.x - button_dim.width() / 2.;
        self.left_button = Button::click(Point { x: x(0.25), y }, left);
        self.right_button = Button::click(Point { x: x(0.75), y }, right);
    }

    fn palette(&self) -> PaletteTemplate {
        self.palettes[self.current_palette]
    }

    fn prev_palette(&mut self) {
        self.current_palette = (self.current_palette + self.palettes.len() - 1) % self.palettes.len();
        self.env.snakes[0].palette = self.palette().into();
    }

    fn next_palette(&mut self) {
        self.current_palette = (self.current_palette + 1) % self.palettes.len();
        self.env.snakes[0].palette = self.palette().into();
    }

    fn snake(&self) -> &Snake {
        &self.env.snakes[0]
    }

    fn snake_mut(&mut self) -> &mut Snake {
        &mut self.env.snakes[0]
    }

    /// Put this demo's snake `offset` cells ahead of `leader`'s in its move
    /// schedule. Only the schedule moves: the snake's body stays where it is.
    fn sync_schedule(&mut self, leader: &SnakeDemo, offset: usize) {
        let position = leader.snake().controller.schedule_position().unwrap_or(0);
        self.snake_mut().controller.set_schedule_position(position + offset);
    }

    /// Take over `leader`'s progress through its current cell and its speed.
    /// Called before both advance by the same time, so they cross cell
    /// boundaries on the same frame and can't drift apart, whatever paths
    /// their schedules trace.
    fn follow(&mut self, leader: &SnakeDemo) {
        let (head_fraction, speed) = (leader.snake().body.head_fraction, leader.snake().speed);
        let snake = self.snake_mut();
        snake.body.head_fraction = head_fraction;
        snake.speed = speed;
    }

    fn update(&mut self, elapsed: Duration) {
        advance_snakes(&mut self.env, elapsed);
        update_snake_dirs(&mut self.env);
    }

    /// Draw the board and snake. Buttons are drawn separately, in screen space.
    fn draw_board(&mut self, material: &Material) -> Result<()> {
        let gtx = &self.env.gtx;
        let grid = rendering::grid_mesh(gtx)?;
        let border = rendering::border_mesh(gtx)?;
        let snake = rendering::snake_mesh(&mut self.env.snakes, &[], gtx, &mut Stats::default())?;

        set_board_camera(self.pos);
        grid.draw();
        for (mesh, _lut) in &snake.shaded {
            mesh.draw_shaded(material);
        }
        border.draw();
        Ok(())
    }

    /// Draw the palette arrows; `interactive` is false while a menu covers
    /// them
    fn draw_buttons(&mut self, interactive: bool) {
        if !interactive {
            self.left_button.draw_idle();
            self.right_button.draw_idle();
            return;
        }
        if self.left_button.draw() {
            self.prev_palette();
        }
        if self.right_button.draw() {
            self.next_palette();
        }
    }
}

/// The screen shown before a game: choose the number of players and each
/// player's palette, then start.
pub struct StartScreen {
    /// One seed per possible player; the first `players` are used
    seeds: Vec<SnakeBuilder>,
    cell_dim: CellDim,
    palette: app::Palette,
    spawn_policy: SpawnPolicy,
    mode: GameMode,

    players_button: Button,
    options_button: Button,
    start_button: Button,
    /// The options menu, over the start screen
    menu: Menu,
    /// The preferences the menu edits (the game loads them when it starts)
    prefs: Prefs,
    demos: Vec<SnakeDemo>,
    /// Compiled lazily on first draw (needs a live GL context)
    snake_material: Option<Material>,

    start_game: bool,
    /// Window size the layout was computed for
    laid_out_for: (f32, f32),
}

impl StartScreen {
    pub fn new(
        cell_dim: CellDim,
        seeds: Vec<SnakeBuilder>,
        palette: app::Palette,
        spawn_policy: SpawnPolicy,
        mode: GameMode,
    ) -> Self {
        assert!(!seeds.is_empty(), "No players specified");

        let button = |shape: &ShapePoints, text: &str| {
            ButtonData::new(shape.clone(), STROKE_THICKNESS, BUTTON_COLOR).text(text, FONT_SIZE, BUTTON_COLOR)
        };

        // wider than the start button to fit "Two players"
        let players_shape = WideHexagon::with_h_side(BUTTON_CELL_DIM, 5. * BUTTON_CELL_DIM.side + 20.);
        let player_options = ["One player", "Two players"][..seeds.len().min(2)]
            .iter()
            .map(|text| button(&players_shape, text))
            .collect();

        let mut demos: Vec<SnakeDemo> = seeds
            .iter()
            .map(|_| SnakeDemo::new(CellDim::from(20.), Point::zero(), palette.clone()))
            .collect();
        // all demos run the same schedule in step
        if let Some((leader, followers)) = demos.split_first_mut() {
            for follower in followers {
                follower.sync_schedule(leader, 0);
            }
        }

        show_mouse(true);

        let mut this = Self {
            seeds,
            cell_dim,
            palette,
            spawn_policy,
            mode,

            // placed by `layout`
            players_button: Button::rotate(Point::zero(), player_options),
            options_button: Button::click(Point::zero(), button(&players_shape, "Options")),
            start_button: Button::click(Point::zero(), button(&WideHexagon::new(BUTTON_CELL_DIM), "Start")),
            menu: Menu::Closed,
            prefs: Prefs::load(),
            demos,
            snake_material: None,

            start_game: false,
            laid_out_for: (0., 0.),
        };
        this.layout();
        this
    }

    fn players(&self) -> usize {
        self.players_button.index() + 1
    }

    const HINT: &'static str = "Enter to start";
    const HINT_SIZE: f32 = 20.;
    /// Space below the hint, and between it and the start button
    const HINT_MARGIN: f32 = 20.;
    const HINT_GAP: f32 = 16.;

    /// Where the hint's text starts, from the top
    fn hint_top(height: f32) -> f32 {
        height - Self::HINT_MARGIN - measure_text(Self::HINT, Self::HINT_SIZE).height
    }

    /// Place everything relative to the window: the players and options
    /// buttons side by side at the top, the demos of the active players side by side in the middle, the
    /// start button at the bottom.
    fn layout(&mut self) {
        let (width, height) = (screen_width(), screen_height());
        self.laid_out_for = (width, height);

        let margin = BUTTON_CELL_DIM.side;
        let button_height = BUTTON_CELL_DIM.height();

        // the two top buttons are the same width, centered together
        let top_width = self.players_button.size().x;
        let top_gap = 2. * margin;
        let top_left = (width - 2. * top_width - top_gap) / 2.;
        self.players_button.pos = Point { x: top_left, y: margin };
        self.options_button.pos = Point { x: top_left + top_width + top_gap, y: margin };

        // the start button sits above the hint
        let start_y = Self::hint_top(height) - Self::HINT_GAP - button_height;
        self.start_button.pos = Point { x: (width - self.start_button.size().x) / 2., y: start_y };

        // the demos fill the space between the buttons, arrows included
        let players = self.players();
        let top = 2. * margin + button_height;
        let bottom = start_y - margin;
        let arrows = BUTTON_CELL_DIM.side + BUTTON_CELL_DIM.height();
        let unit = SnakeDemo::board_size(CellDim::from(1.));
        let side = ((width - margin * (players + 1) as f32) / players as f32 / unit.x)
            .min((bottom - top - arrows) / unit.y)
            .max(1.);
        let cell_dim = CellDim::from(side);
        let board = SnakeDemo::board_size(cell_dim);

        let total_width = players as f32 * board.x + (players - 1) as f32 * margin;
        let left = (width - total_width) / 2.;
        let y = top + (bottom - top - arrows - board.y) / 2.;
        for (i, demo) in self.demos.iter_mut().enumerate() {
            let x = left + i as f32 * (board.x + margin);
            demo.layout(cell_dim, Point { x, y });
        }
    }
}

impl Screen for StartScreen {
    fn update(&mut self) -> Result<()> {
        // capped so a stalled frame doesn't send the snakes flying
        let elapsed = Duration::from_secs_f32(get_frame_time().min(0.1));

        // Hidden demos keep moving too, so a player's demo is already in step
        // when it appears. The first demo leads; the others follow its timing.
        let (leader, followers) = self.demos.split_first_mut().expect("at least one demo");
        for follower in followers.iter_mut() {
            follower.follow(leader);
        }
        leader.update(elapsed);
        for follower in followers {
            follower.update(elapsed);
        }
        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        if self.laid_out_for != (screen_width(), screen_height()) {
            self.layout();
        }
        if self.snake_material.is_none() {
            self.snake_material = Some(snake_material()?);
        }
        let material = self.snake_material.as_ref().unwrap();

        clear_background(self.palette.background_color);

        let players = self.players();
        for demo in &mut self.demos[..players] {
            demo.draw_board(material)?;
        }

        set_default_camera();
        // under the menu, the buttons are only a picture
        let interactive = !self.menu.is_open();
        for demo in &mut self.demos[..players] {
            demo.draw_buttons(interactive);
        }
        if interactive {
            if self.players_button.draw() {
                // the demos rearrange for the new number of players
                self.layout();
            }
            if self.options_button.draw() {
                self.menu.open();
            }
            if self.start_button.draw() {
                self.start_game = true;
            }
        } else {
            self.players_button.draw_idle();
            self.options_button.draw_idle();
            self.start_button.draw_idle();
        }

        let dims = measure_text(Self::HINT, Self::HINT_SIZE);
        draw_text(
            Self::HINT,
            (screen_width() - dims.width) / 2.,
            Self::hint_top(screen_height()) + dims.offset_y,
            Self::HINT_SIZE,
            Color::new(0.5, 0.5, 0.5, 1.),
        );

        // only the options that are preferences: the others belong to a game
        let options: Vec<_> = Toggle::PREFS
            .iter()
            .map(|&toggle| (toggle, toggle.label(&self.prefs)))
            .collect();
        self.menu.draw(&options, false, &mut self.prefs);

        Ok(())
    }

    fn key_down_event(&mut self, press: KeyPress) -> Result<()> {
        if self.menu.key_pressed(press, &mut self.prefs).0 || self.menu.is_open() {
            return Ok(());
        }
        match press.code {
            KeyCode::Escape => self.menu.open(),
            KeyCode::Enter => self.start_game = true,
            KeyCode::Left => self.demos[0].prev_palette(),
            KeyCode::Right => self.demos[0].next_palette(),
            _ => {}
        }
        Ok(())
    }

    fn transition(&mut self) -> Option<Transition> {
        if !self.start_game {
            return None;
        }
        self.start_game = false;

        let players = self.players();
        let seeds = self
            .seeds
            .iter()
            .zip(&self.demos)
            .take(players)
            .map(|(seed, demo)| {
                let mut seed = seed.clone().palette(demo.palette());
                // a single player uses whichever keys the preferences pick
                if players == 1 {
                    if let Some(Template::Keyboard { side, .. }) = &mut seed.controller {
                        *side = None;
                    }
                }
                seed
            })
            .collect();

        Some(Transition::Push(Box::new(Game::new(
            self.cell_dim,
            seeds,
            self.palette.clone(),
            self.spawn_policy.clone(),
            self.mode,
        ))))
    }

    /// Back from a game: the game may have changed the preferences
    fn resume(&mut self) {
        self.prefs = Prefs::load();
        self.menu = Menu::Closed;
        show_mouse(true);
    }
}
