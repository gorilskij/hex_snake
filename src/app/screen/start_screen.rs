use std::collections::VecDeque;

use ggez::{
    event::EventHandler,
    graphics::{clear, draw, present, DrawParam},
    Context, GameResult,
};

use crate::{
    app::{
        screen::{
            control::Control, prefs::Prefs, rendering::snake_mesh::get_snake_mesh, stats::Stats,
        },
        snake::{
            controller::{programmed::Move, ControllerTemplate},
            palette::PaletteTemplate,
            utils::OtherSnakes,
            Body, EatBehavior, EatMechanics, Segment, SegmentType, Snake, SnakeType, State,
        },
        Screen,
    },
    basic::{CellDim, Dir, HexDim, HexPoint},
};
use ggez::graphics::Color;
use std::slice;
use crate::app::snake::SnakeSeed;

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
        let start_dir = Dir::U;
        let start_pos = top_left + HexPoint { h: 0, v: -5 };
        let board_dim = HexPoint { h: 50, v: 50 };
        let seed = SnakeSeed {
            snake_type: SnakeType::Simulated {
                start_pos,
                start_dir,
                start_grow: 5,
            },
            eat_mechanics: EatMechanics {
                eat_self: EatBehavior::Cut,
                eat_other: hash_map! {},
                default: EatBehavior::Cut,
            },
            palette: PaletteTemplate::Solid {
                color: Color::WHITE,
                eaten: Color::RED,
            }.into(),
            controller: ControllerTemplate::demo_infinity_pattern(1),
        };

        Self {
            control: Control::new(7.),
            top_left,
            dim: board_dim,
            cell_dim,
            snake: Snake::from_seed(&seed, start_pos, start_dir, 5),
            prefs: Default::default(),
            stats: Default::default(),
        }
    }
}

impl SnakeDemo {
    fn advance_snakes(&mut self) {
        self.snake.advance(
            OtherSnakes::empty(),
            &[],
            self.dim,
            self.control.frame_stamp(),
        )
    }
}

impl EventHandler<ggez::GameError> for SnakeDemo {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        while self.control.can_update() {
            self.advance_snakes();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.control.graphics_frame();

        self.snake.update_dir(
            OtherSnakes::empty(),
            &[],
            self.dim,
            self.control.frame_stamp(),
        );
        let mesh = get_snake_mesh(
            slice::from_mut(&mut self.snake),
            &self.control,
            self.dim,
            self.cell_dim,
            self.prefs.draw_style,
            ctx,
            &mut self.stats,
        )?;
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

            player1_demo: SnakeDemo::new(HexPoint { h: 10, v: 10 }, CellDim::from(30.)),
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
