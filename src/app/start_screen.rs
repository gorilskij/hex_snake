use ggez::{Context, GameResult};
use ggez::event::EventHandler;
use ggez::graphics::{BLACK, clear, draw, DrawParam, MeshBuilder, present};

use crate::app::hex::{Dir, Hex, HexPos, HexType};
use crate::app::Screen;
use crate::app::snake::{draw_cell, Snake, SnakeState};
use crate::app::snake::demo_controller::{DemoController, SimMove};

use super::palette::SnakePalette;

struct SnakeDemo {
    top_left: HexPos,
    palette: SnakePalette,
    snake: Snake<DemoController>,
}

impl SnakeDemo {
    fn new(top_left: HexPos) -> Self {
        let pos = top_left + HexPos { h: 0, v: -5 };
        Self {
            top_left,
            palette: SnakePalette::gray_gradient(),
            snake: Snake {
                body: vec![Hex {
                    typ: HexType::Normal,
                    pos,
                    teleported: None,
                }],
                palette: SnakePalette::gray_gradient(),
                state: SnakeState::Living,
                dir: Dir::U,
                grow: 10,
                controller: DemoController::new(vec![
                    SimMove::Move(Dir::UR),
                    SimMove::Wait(5),
                    SimMove::Move(Dir::DR),
                    SimMove::Wait(5),
                    SimMove::Move(Dir::D),
                    SimMove::Wait(5),
                    SimMove::Move(Dir::DL),
                    SimMove::Wait(5),
                    SimMove::Move(Dir::UL),
                    SimMove::Wait(5),
                ]),
                game_dim: HexPos { h: 20, v: 20 },
            },
        }
    }
}

impl EventHandler for SnakeDemo {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.snake.advance();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut builder = MeshBuilder::new();
        let mut dc = |h, v, c, dir| draw_cell(&mut builder, h, v, c, dir);
        self.snake.draw_non_crash_points(&mut dc)?;
        let mesh = &builder.build(ctx)?;
        draw(ctx, mesh, DrawParam::default())?;
        Ok(())
    }
}

pub struct StartScreen {
    // options selected
    players: usize,
    palettes: Vec<SnakePalette>,
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
                SnakePalette::gray_gradient(),
                SnakePalette::rainbow(),
                SnakePalette::rainbow_sin(10),
            ],
            player1_palette_idx: 0,
            player2_palette_idx: 0,

            player1_demo: SnakeDemo::new(HexPos { h: 10, v: 10 }),
            player2_demo: None,
        }
    }
}

impl EventHandler for StartScreen {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.player1_demo.update(ctx)?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        clear(ctx, BLACK);

        self.player1_demo.draw(ctx)?;

        present(ctx)
    }
}

impl StartScreen {
    pub fn next_screen(&self) -> Option<Screen> {
        None
    }
}