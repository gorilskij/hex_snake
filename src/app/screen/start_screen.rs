use std::collections::VecDeque;

use ggez::{
    event::EventHandler,
    graphics::{clear, draw, present, DrawParam, MeshBuilder},
    Context, GameResult,
};

use crate::{
    app::{
        snake::{palette::PaletteTemplate, Segment, SegmentType, Snake, SnakeType, controller::{ControllerTemplate, programmed::Move}},
        Screen,
    },
    basic::{CellDim, Dir, HexDim, HexPoint},
};
use ggez::graphics::Color;
use crate::app::snake::{EatMechanics, EatBehavior, Body, State};
use crate::app::screen::rendering::snake_mesh::get_snake_mesh;
use std::slice;
use crate::app::screen::prefs::Prefs;
use crate::app::screen::stats::Stats;
use crate::app::screen::control::Control;
use crate::app::snake::utils::OtherSnakes;

struct SnakeDemo {
    control: Control,
    top_left: HexPoint,
    dim: HexDim,
    cell_dim: CellDim,
    snake: Snake,
    prefs: Prefs,
    stats: Stats,
}

impl SnakeDemo {
    fn new(top_left: HexPoint, cell_dim: CellDim) -> Self {
        let dir = Dir::U;

        let pos = top_left + HexPoint { h: 0, v: -5 };
        let head = Segment {
            typ: SegmentType::Normal,
            pos,
            coming_from: -dir,
            teleported: None,
        };
        let mut body = VecDeque::new();
        body.push_back(head);
        let board_dim = HexPoint { h: 20, v: 20 };

        Self {
            control: Control::new(3.),
            top_left,
            dim: board_dim,
            cell_dim,
            snake: Snake {
                snake_type: SnakeType::Simulated {
                    start_pos: HexPoint { h: 10, v: 10 },
                    start_dir: Dir::U,
                    start_grow: 5,
                },
                eat_mechanics: EatMechanics {
                    eat_self: EatBehavior::Cut,
                    eat_other: hash_map! {},
                    default: EatBehavior::Cut,
                },

                body: Body {
                    cells: body,
                    missing_front: 0,
                    dir,
                    turn_start: None,
                    dir_grace: false,
                    grow: 10,
                    search_trace: None
                },
                state: State::Living,

                controller: ControllerTemplate::Programmed(vec![
                    Move::Turn(Dir::Ur),
                    Move::Wait(5),
                    Move::Turn(Dir::Dr),
                    Move::Wait(5),
                    Move::Turn(Dir::D),
                    Move::Wait(5),
                    Move::Turn(Dir::Dl),
                    Move::Wait(5),
                    Move::Turn(Dir::Ul),
                    Move::Wait(5),
                ])
                .into_controller(dir),
                palette: PaletteTemplate::Solid { color: Color::WHITE, eaten: Color::RED }.into(),
            },
            prefs: Default::default(),
            stats: Default::default(),
        }
    }
}

impl EventHandler<ggez::GameError> for SnakeDemo {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // self.snake.advance(OtherSnakes::empty(), &[], self.dim);
        self.snake.advance(OtherSnakes::empty(), &[], self.dim, self.control.frame_stamp());
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.snake.update_dir(OtherSnakes::empty(), &[], self.dim, self.control.frame_stamp());
        let mesh = get_snake_mesh(slice::from_mut(&mut self.snake), &self.control, self.dim, self.cell_dim, self.prefs.draw_style, ctx, &mut self.stats)?;
        draw(ctx, &mesh, DrawParam::default())?;
        Ok(())
    }
}

pub struct StartScreen {
    players: usize,
    palettes: Vec<PaletteTemplate>,
    player1_palette_idx: usize,
    player2_palette_idx: usize,

    player1_demo: SnakeDemo,
    player2_demo: Option<SnakeDemo>,
}

impl StartScreen {
    pub fn new() -> Self {
        Self {
            players: 1,
            palettes: vec![
                PaletteTemplate::gray_gradient(false),
                PaletteTemplate::rainbow(false),
                // SnakePaletteTemplate::new_rainbow_sin(10),
            ],
            player1_palette_idx: 0,
            player2_palette_idx: 0,

            player1_demo: SnakeDemo::new(HexPoint { h: 10, v: 10 }, CellDim::from(10.)),
            player2_demo: None,
        }
    }
}

impl EventHandler<ggez::GameError> for StartScreen {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.player1_demo.update(ctx)?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        clear(ctx, Color::BLACK);

        self.player1_demo.draw(ctx)?;

        present(ctx)
    }
}

impl StartScreen {
    pub fn next_screen(&self) -> Option<Screen> {
        None
    }
}
