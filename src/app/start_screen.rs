use std::collections::VecDeque;

use ggez::{
    event::EventHandler,
    graphics::{clear, draw, present, DrawParam, MeshBuilder},
    Context, GameResult,
};

use crate::{
    app::{
        snake::{
            controller::OtherSnakes, palette::SnakePaletteTemplate, Segment, SegmentType, Snake,
        },
        Screen,
    },
    basic::{CellDim, Dir, HexDim, HexPoint},
};
use ggez::graphics::Color;

struct SnakeDemo {
    top_left: HexPoint,
    dim: HexDim,
    snake: Snake,
    cell_dim: CellDim,
}

impl SnakeDemo {
    fn new(top_left: HexPoint, cell_dim: CellDim) -> Self {
        let dir = Dir::U;

        let pos = top_left + HexPoint { h: 0, v: -5 };
        let head = Segment {
            typ: SegmentType::Normal,
            pos,
            next_segment: -dir,
            teleported: None,
        };
        let mut body = VecDeque::new();
        body.push_back(head);
        let board_dim = HexPoint { h: 20, v: 20 };
        todo!()
        // Self {
        //     top_left,
        //     dim: board_dim,
        //     snake: Snake {
        //         snake_type: SnakeType::SimulatedSnake,
        //         eat_mechanics: EatMechanics {
        //             eat_self: EatBehavior::Cut,
        //             eat_other: hash_map! {},
        //             default: EatBehavior::Cut,
        //         },
        //
        //         body: SnakeBody {
        //             cells: body,
        //             dir,
        //             dir_grace: false,
        //             grow: 10,
        //         },
        //         state: SnakeState::Living,
        //
        //         controller: SnakeControllerTemplate::DemoController(vec![
        //             SimMove::Move(Dir::UR),
        //             SimMove::Wait(5),
        //             SimMove::Move(Dir::DR),
        //             SimMove::Wait(5),
        //             SimMove::Move(Dir::D),
        //             SimMove::Wait(5),
        //             SimMove::Move(Dir::DL),
        //             SimMove::Wait(5),
        //             SimMove::Move(Dir::UL),
        //             SimMove::Wait(5),
        //         ])
        //         .into_controller(dir),
        //         painter: SnakePaletteTemplate::gray_gradient().into(),
        //     },
        //     cell_dim,
        // }
    }
}

impl EventHandler for SnakeDemo {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.snake.advance(OtherSnakes(&[], &[]), &[], self.dim);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let builder = &mut MeshBuilder::new();
        // todo!();
        // self.snake.draw_non_crash_points(builder, self.cell_dim)?;
        let mesh = builder.build(ctx)?;
        draw(ctx, &mesh, DrawParam::default())?;
        Ok(())
    }
}

pub struct StartScreen {
    // options selected
    players: usize,
    palettes: Vec<SnakePaletteTemplate>,
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
                SnakePaletteTemplate::gray_gradient(),
                SnakePaletteTemplate::rainbow(),
                // SnakePaletteTemplate::new_rainbow_sin(10),
            ],
            player1_palette_idx: 0,
            player2_palette_idx: 0,

            player1_demo: SnakeDemo::new(HexPoint { h: 10, v: 10 }, CellDim::from(10.)),
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
