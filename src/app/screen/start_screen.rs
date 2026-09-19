use std::f32::consts::TAU;
use std::time::Duration;

use anyhow::Result;
use enum_map_lite::enum_map;
use macroquad::camera::set_default_camera;
use macroquad::color::Color;
use macroquad::input::{show_mouse, KeyCode};
use macroquad::material::Material;
use macroquad::window::{clear_background, screen_height, screen_width};
use rand::prelude::*;

use super::Game;
use crate::app::game_context::GameContext;
use crate::app::fps_control::FpsControl;
use crate::app::game_mode::GameMode;
use crate::app::key::Key;
use crate::app::prefs::{DrawGrid, Prefs};
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
use crate::support::mesh::{set_board_camera, Mesh};

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

    /// The grid and border, built once for the settings and cell size they
    /// were built for (only the snake changes from frame to frame)
    board_meshes: Option<(BoardKey, Option<Mesh>, Option<Mesh>)>,
}

/// What the demo's grid and border depend on
#[derive(Copy, Clone, PartialEq)]
struct BoardKey {
    side: f32,
    grid: DrawGrid,
    border: bool,
}

impl SnakeDemo {
    /// The center of the snake's hexagonal loop, and how far the loop is
    /// from it; the board is just big enough for the drawn cells around it
    const CENTER: HexPoint = HexPoint { h: 4, v: 4 };
    const LOOP_RADIUS: usize = 3;
    const BOARD_DIM: HexDim = HexPoint { h: 9, v: 9 };
    /// Space between the board and the arrow buttons, relative to a button's
    /// side
    const ARROW_GAP: f32 = 0.4;

    fn new(cell_dim: CellDim, pos: Point, app_palette: app::Palette) -> Self {
        let palettes = palettes();

        // A loop of 6 sides of LOOP_RADIUS cells around CENTER, longer than
        // the snake, so it never runs into itself. It starts at the bottom-left
        // corner, going up.
        let snake = SnakeBuilder::default()
            .snake_type(snake::Type::Simulated)
            .eat_mechanics(EatMechanics::new(
                enum_map! { _ => EatBehavior::Crash },
                enum_map! { _ => enum_map! { _ => EatBehavior::Crash } },
            ))
            .controller(Template::demo_hexagon_pattern(Dir::U, Self::LOOP_RADIUS - 1))
            .pos(Self::CENTER.translate(Dir::Dl, Self::LOOP_RADIUS))
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
            board_meshes: None,
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

    /// The board's cells that are drawn: a hexagon one cell wider on every
    /// side than the snake's loop
    fn cells() -> impl Iterator<Item = HexPoint> {
        let HexDim { h, v } = Self::BOARD_DIM;
        (0..h)
            .flat_map(move |h| (0..v).map(move |v| HexPoint { h, v }))
            .filter(|&pos| Self::shown(pos))
    }

    fn shown(pos: HexPoint) -> bool {
        pos.manhattan_distance(Self::CENTER) <= Self::LOOP_RADIUS + 1
    }

    /// Top-left and bottom-right of the drawn cells, in board coordinates
    fn bounds(cell_dim: CellDim) -> (Point, Point) {
        let size = Point { x: cell_dim.width(), y: cell_dim.height() };
        Self::cells().map(|pos| pos.to_cartesian(cell_dim)).fold(
            (Point { x: f32::MAX, y: f32::MAX }, Point { x: f32::MIN, y: f32::MIN }),
            |(min, max), p| {
                (
                    Point { x: min.x.min(p.x), y: min.y.min(p.y) },
                    Point { x: max.x.max(p.x + size.x), y: max.y.max(p.y + size.y) },
                )
            },
        )
    }

    /// Size of the demo board (without the buttons) for a given cell size
    fn board_size(cell_dim: CellDim) -> Point {
        let (min, max) = Self::bounds(cell_dim);
        max - min
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

        let y = pos.y + board.y + Self::ARROW_GAP * button_dim.side;
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
        self.set_palette(self.current_palette + 1);
    }

    /// Show the `index`th palette (wrapping around the list)
    fn set_palette(&mut self, index: usize) {
        self.current_palette = index % self.palettes.len();
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

    /// The grid and border, rebuilt only when what they depend on changes
    fn board_meshes(&mut self) -> (Option<&Mesh>, Option<&Mesh>) {
        let gtx = &self.env.gtx;
        let key = BoardKey {
            side: gtx.cell_dim.side,
            grid: gtx.prefs.draw_grid,
            border: gtx.prefs.draw_border,
        };
        if self.board_meshes.as_ref().is_none_or(|(built, ..)| *built != key) {
            let palette = &gtx.palette;
            let grid = match key.grid {
                DrawGrid::Grid => Some(rendering::region_grid_mesh(
                    Self::cells(),
                    Self::shown,
                    gtx.cell_dim,
                    palette.grid_thickness,
                    palette.grid_color,
                )),
                DrawGrid::Dots => Some(rendering::region_dot_mesh(
                    Self::cells(),
                    gtx.cell_dim,
                    palette.grid_dot_radius,
                    palette.grid_dot_color,
                )),
                DrawGrid::None => None,
            };
            let border = key.border.then(|| {
                rendering::region_border_mesh(
                    Self::cells(),
                    Self::shown,
                    gtx.cell_dim,
                    palette.border_thickness,
                    palette.border_color,
                )
            });
            self.board_meshes = Some((key, grid, border));
        }
        let (_, grid, border) = self.board_meshes.as_ref().expect("just built");
        (grid.as_ref(), border.as_ref())
    }

    /// Draw the board and snake. Buttons are drawn separately, in screen space.
    fn draw_board(&mut self, material: &Material) -> Result<()> {
        let offset = self.pos - Self::bounds(self.env.gtx.cell_dim).0;
        let snake = rendering::snake_mesh(&mut self.env.snakes, &[], &self.env.gtx, &mut Stats::default())?;
        let (grid, border) = self.board_meshes();

        // the drawn cells start at `pos`
        set_board_camera(offset);
        if let Some(grid) = grid {
            grid.draw();
        }
        for (mesh, _lut) in &snake.shaded {
            mesh.draw_shaded(material);
        }
        if let Some(border) = border {
            border.draw();
        }
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
    /// Evens out the frame durations the demos advance by
    frame_pace: FpsControl,
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
            .enumerate()
            .map(|(i, _)| {
                let mut demo = SnakeDemo::new(CellDim::from(20.), Point::zero(), palette.clone());
                // each player starts out with a different palette
                demo.set_palette(i);
                demo
            })
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
            frame_pace: FpsControl::new(),
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
        // space above the demos, below them (under the arrows), and between
        let (above, below) = (100., 60.);
        let demo_gap = 2. * margin;
        let top = margin + button_height + above;
        let bottom = start_y - below;
        let arrows = SnakeDemo::ARROW_GAP * BUTTON_CELL_DIM.side + BUTTON_CELL_DIM.height();
        let unit = SnakeDemo::board_size(CellDim::from(1.));
        let side = ((width - 2. * margin - demo_gap * (players - 1) as f32) / players as f32 / unit.x)
            .min((bottom - top - arrows) / unit.y)
            .max(1.);
        let cell_dim = CellDim::from(side);
        let board = SnakeDemo::board_size(cell_dim);

        let total_width = players as f32 * board.x + (players - 1) as f32 * demo_gap;
        let left = (width - total_width) / 2.;
        let y = top + (bottom - top - arrows - board.y) / 2.;
        for (i, demo) in self.demos.iter_mut().enumerate() {
            let x = left + i as f32 * (board.x + demo_gap);
            demo.layout(cell_dim, Point { x, y });
        }
    }
}

impl Screen for StartScreen {
    fn update(&mut self) -> Result<()> {
        // smoothed like the game's, so the demos move as evenly as the game
        // (the web's clock only has whole milliseconds)
        let Some(elapsed) = self.frame_pace.update() else {
            return Ok(());
        };

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
            // the demos show the board and snakes as the game will
            let prefs = &mut demo.env.gtx.prefs;
            prefs.draw_style = self.prefs.draw_style;
            prefs.draw_grid = self.prefs.draw_grid;
            prefs.draw_border = self.prefs.draw_border;
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
            .map(|&toggle| vec![(toggle, toggle.label(&self.prefs))])
            .collect();
        self.menu.draw(&options, false, &mut self.prefs);

        Ok(())
    }

    fn key_down_event(&mut self, key: Key) -> Result<()> {
        if self.menu.key_pressed(key, &mut self.prefs).0 || self.menu.is_open() {
            return Ok(());
        }
        match key {
            Key::Code(KeyCode::Escape) => self.menu.open(),
            Key::Code(KeyCode::Enter) => self.start_game = true,
            // with two players it would be unclear whose palette the keys
            // change: only the on-screen buttons work then
            Key::Code(KeyCode::Left) if self.players() == 1 => self.demos[0].prev_palette(),
            Key::Code(KeyCode::Right) if self.players() == 1 => self.demos[0].next_palette(),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The snake runs its loop at exactly the loop's radius from the center,
    /// so the drawn hexagon is one cell wider than the loop all round.
    #[test]
    fn the_demo_snake_loops_around_the_center() {
        let mut demo = SnakeDemo::new(CellDim::from(10.), Point::zero(), app::Palette::dark());
        let mut visited = std::collections::HashSet::new();
        // well over a few laps, in small steps
        for _ in 0..2000 {
            demo.update(Duration::from_secs_f32(0.01));
            let snake = &demo.env.snakes[0];
            for segment in &snake.body.segments {
                visited.insert(segment.pos);
            }
        }
        assert_eq!(visited.len(), 6 * SnakeDemo::LOOP_RADIUS, "one cell per step of the loop");
        for pos in visited {
            assert_eq!(pos.manhattan_distance(SnakeDemo::CENTER), SnakeDemo::LOOP_RADIUS, "{pos:?}");
            assert!(SnakeDemo::BOARD_DIM.contains(pos));
        }
    }

    /// The whole hexagon fits on the board: none of it is cut off.
    #[test]
    fn the_drawn_hexagon_is_whole() {
        let radius = SnakeDemo::LOOP_RADIUS + 1;
        assert_eq!(SnakeDemo::cells().count(), 1 + 3 * radius * (radius + 1));
    }
}
