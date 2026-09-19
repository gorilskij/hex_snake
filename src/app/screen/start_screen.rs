use std::f32::consts::TAU;
use std::time::Duration;

use anyhow::Result;
use enum_map_lite::enum_map;
use macroquad::camera::set_default_camera;
use macroquad::color::Color;
use macroquad::input::{show_mouse, KeyCode};
use macroquad::material::Material;
use macroquad::text::{draw_text, measure_text};
use macroquad::time::get_frame_time;
use macroquad::window::{clear_background, screen_height, screen_width};
use rand::prelude::*;

use super::Game;
use crate::app::game_context::GameContext;
use crate::app::game_mode::GameMode;
use crate::app::prefs::Prefs;
use crate::app::screen::{Environment, Screen};
use crate::app::snake_management::{advance_snakes, update_snake_dirs};
use crate::app::stats::Stats;
use crate::app::{self};
use crate::apple::spawn::SpawnPolicy;
use crate::basic::{CellDim, Dir, HexDim, HexPoint, Point};
use crate::button::{Button, ButtonData, TriColor};
use crate::rendering;
use crate::rendering::shape::{Hexagon, Shape, ShapePoints, TriangleArrowLeft, WideHexagon};
use crate::snake::builder::Builder as SnakeBuilder;
use crate::snake::eat_mechanics::{EatBehavior, EatMechanics};
use crate::snake::{self, PaletteTemplate, Snake};
use crate::snake_control::Template;
use crate::support::material::snake_material;
use crate::support::mesh::set_board_camera;

const BUTTON_COLOR: TriColor = TriColor {
    normal: Color::new(0.5, 0.5, 0.5, 1.),
    hover: Color::new(0., 1., 0., 1.),
    pressed: Color::new(1., 0., 0., 1.),
};
const STROKE_THICKNESS: f32 = 4.;
const FONT_SIZE: f32 = 32.;
/// Cell size of the menu's buttons, independent of the window size
const BUTTON_CELL_DIM: CellDim = CellDim { side: 30., sin: 25.980762, cos: 15. };

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

        let outer_shape = Hexagon::new(BUTTON_CELL_DIM);
        let inner_shape = TriangleArrowLeft::new(BUTTON_CELL_DIM * 0.6);
        let arrow_offset = outer_shape.center() - inner_shape.center();
        let left = ButtonData::new(outer_shape.clone(), STROKE_THICKNESS, BUTTON_COLOR).inner_shape(
            inner_shape.clone(),
            arrow_offset,
            STROKE_THICKNESS,
            BUTTON_COLOR,
        );
        let right = ButtonData::new(outer_shape, STROKE_THICKNESS, BUTTON_COLOR).inner_shape(
            inner_shape.rotate_clockwise_about_center(TAU / 2.),
            arrow_offset,
            STROKE_THICKNESS,
            BUTTON_COLOR,
        );

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
            // placed by `layout`
            left_button: Button::click(Point::zero(), left),
            right_button: Button::click(Point::zero(), right),
        };
        this.layout(cell_dim, pos);
        this
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
        let y = pos.y + board.y + BUTTON_CELL_DIM.side;
        self.left_button.pos = Point { x: pos.x, y };
        self.right_button.pos = Point {
            x: pos.x + board.x - BUTTON_CELL_DIM.width(),
            y,
        };
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

    fn draw_buttons(&mut self) {
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
    spawn_policy: Option<SpawnPolicy>,
    mode: GameMode,

    players_button: Button,
    start_button: Button,
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
            spawn_policy: Some(spawn_policy),
            mode,

            // placed by `layout`
            players_button: Button::rotate(Point::zero(), player_options),
            start_button: Button::click(Point::zero(), button(&WideHexagon::new(BUTTON_CELL_DIM), "Start")),
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

    /// Place everything relative to the window: the players button at the
    /// top, the demos of the active players side by side in the middle, the
    /// start button at the bottom.
    fn layout(&mut self) {
        let (width, height) = (screen_width(), screen_height());
        self.laid_out_for = (width, height);

        let margin = BUTTON_CELL_DIM.side;
        let button_height = BUTTON_CELL_DIM.height();
        let centered = |button: &Button, y: f32| Point { x: (width - button.size().x) / 2., y };

        self.players_button.pos = centered(&self.players_button, margin);
        self.start_button.pos = centered(&self.start_button, height - margin - button_height);

        // the demos fill the space between the two buttons, arrows included
        let players = self.players();
        let top = 2. * margin + button_height;
        let bottom = height - 2. * margin - button_height;
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
        for demo in &mut self.demos[..players] {
            demo.draw_buttons();
        }
        if self.players_button.draw() {
            // the demos rearrange for the new number of players
            self.layout();
        }
        if self.start_button.draw() {
            self.start_game = true;
        }

        let hint = "Enter to start";
        let dims = measure_text(hint, None, 24, 1.);
        draw_text(
            hint,
            (screen_width() - dims.width) / 2.,
            screen_height() - 10.,
            24.,
            Color::new(0.5, 0.5, 0.5, 1.),
        );

        Ok(())
    }

    fn key_down_event(&mut self, keycode: KeyCode) -> Result<()> {
        match keycode {
            KeyCode::Enter => self.start_game = true,
            KeyCode::Left => self.demos[0].prev_palette(),
            KeyCode::Right => self.demos[0].next_palette(),
            _ => {}
        }
        Ok(())
    }

    fn next_screen(&mut self) -> Option<Box<dyn Screen>> {
        if !self.start_game {
            return None;
        }

        let players = self.players();
        let seeds = self
            .seeds
            .iter()
            .zip(&self.demos)
            .take(players)
            .map(|(seed, demo)| seed.clone().palette(demo.palette()))
            .collect();

        Some(Box::new(Game::new(
            self.cell_dim,
            seeds,
            self.palette.clone(),
            self.spawn_policy.take()?,
            self.mode,
        )))
    }
}
