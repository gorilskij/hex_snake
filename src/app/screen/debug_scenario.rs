use crate::basic::{HexDim, HexPoint, Dir, Point, CellDim};
use crate::app::screen::Apple;
use crate::app::apple_spawn_strategy::AppleSpawnStrategy;
use crate::app::snake::{self, Snake, EatMechanics, EatBehavior};
use crate::app::snake::controller::ControllerTemplate;
use ggez::graphics::{Color, clear, present, DrawParam, draw};
use ggez::event::EventHandler;
use ggez::{Context, GameError, GameResult};
use crate::app::screen::control::{Control, FrameStamp};
use crate::app::snake::utils::split_snakes_mut;
use crate::app::collisions::{find_collisions, handle_collisions};
use crate::app::screen::rendering::grid_mesh::{get_grid_mesh, get_border_mesh};
use crate::app::screen::rendering::snake_mesh::get_snake_mesh;
use crate::app::screen::rendering::apple_mesh::get_apple_mesh;
use crate::app;
use crate::app::screen::prefs::Prefs;
use crate::app::screen::stats::Stats;

pub struct DebugScenario {
    control: Control,

    cell_dim: CellDim,

    board_dim: HexDim,
    offset: Point,
    fit_to_window: bool,

    palette: app::Palette,
    prefs: Prefs,
    stats: Stats,

    apples: Vec<Apple>,
    apple_spawn_strategy: AppleSpawnStrategy,

    seeds: Vec<snake::Seed>,
    snakes: Vec<Snake>,
}

impl DebugScenario {
    /// A snake crashes into another snake's body
    pub fn collision1(cell_dim: CellDim) -> Self {
        // snake2 crashes into snake1

        let snake1_start_pos = HexPoint { h: 5, v: 7 };
        let snake1_start_dir = Dir::U;
        let snake1_start_grow = 5;

        let snake2_start_pos = HexPoint { h: 8, v: 7 };
        let snake2_start_dir = Dir::Ul;
        let snake2_start_grow = 5;

        let seed1 = snake::Seed {
            snake_type: snake::Type::Simulated {
                start_pos: snake1_start_pos,
                start_dir: snake1_start_dir,
                start_grow: snake1_start_grow,
            },
            eat_mechanics: EatMechanics {
                eat_self: EatBehavior::Crash,
                eat_other: hash_map![],
                default: EatBehavior::Crash,
            },
            palette: snake::PaletteTemplate::solid_white_red(),
            controller: ControllerTemplate::Programmed(vec![]),
        };

        let seed2 = snake::Seed {
            snake_type: snake::Type::Simulated {
                start_pos: snake2_start_pos,
                start_dir: snake2_start_dir,
                start_grow: snake2_start_grow,
            },
            eat_mechanics: EatMechanics {
                eat_self: EatBehavior::Crash,
                eat_other: hash_map![],
                default: EatBehavior::Crash,
            },
            palette: snake::PaletteTemplate::Solid { color: Color::RED, eaten: Color::WHITE },
            controller: ControllerTemplate::Programmed(vec![]),
        };

        let snake1 = Snake::from_seed(&seed1, snake1_start_pos, snake1_start_dir, snake1_start_grow);
        let snake2 = Snake::from_seed(&seed2, snake2_start_pos, snake2_start_dir, snake2_start_grow);

        Self {
            control: Control::new(7.),

            cell_dim,

            board_dim: HexDim { h: 20, v: 10 },
            offset: Point { x: 0., y: 0. },
            fit_to_window: false,

            palette: app::Palette::dark(),
            prefs: Default::default(),
            stats: Default::default(),

            apples: vec![],
            apple_spawn_strategy: AppleSpawnStrategy::None,

            seeds: vec![seed1, seed2],
            snakes: vec![snake1, snake2],
        }
    }

    fn spawn_apples(&mut self) {
        // TODO
    }

    fn advance_snakes(&mut self, frame_stamp: FrameStamp) {
        for snake_index in 0..self.snakes.len() {
            let (snake, other_snakes) = split_snakes_mut(&mut self.snakes, snake_index);
            snake.advance(other_snakes, &self.apples, self.board_dim, frame_stamp);
        }

        let collisions = find_collisions(&self.snakes, &self.apples);
        let (spawn_snakes, remove_apples, game_over) =
            handle_collisions(&collisions, &mut self.snakes, &self.apples);

        if game_over {
            self.control.game_over();
        }

        assert!(spawn_snakes.is_empty(), "unimplemented");

        for apple_index in remove_apples.into_iter().rev() {
            self.apples.remove(apple_index);
        }

        self.spawn_apples();
    }
}

impl EventHandler<ggez::GameError> for DebugScenario {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        while self.control.can_update() {
            let frame_stamp = self.control.frame_stamp();
            self.advance_snakes(frame_stamp);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.control.graphics_frame();
        let frame_stamp = self.control.frame_stamp();

        clear(ctx, Color::BLACK);

        let draw_param = DrawParam::default().dest(self.offset);

        let grid_mesh = get_grid_mesh(self.board_dim, self.cell_dim, &self.palette, ctx)?;
        draw(ctx, &grid_mesh, draw_param)?;

        let border_mesh = get_border_mesh(self.board_dim, self.cell_dim, &self.palette, ctx)?;
        draw(ctx, &border_mesh, draw_param)?;

        for snake_index in 0..self.snakes.len() {
            let (snake, other_snakes) = split_snakes_mut(&mut self.snakes, snake_index);
            snake.update_dir(other_snakes, &self.apples, self.board_dim, frame_stamp);
        }

        let snake_mesh = get_snake_mesh(
            &mut self.snakes,
            frame_stamp,
            self.board_dim,
            self.cell_dim,
            self.prefs.draw_style,
            ctx,
            &mut self.stats,
        )?;
        draw(ctx, &snake_mesh, draw_param)?;

        if !self.apples.is_empty() {
            let apple_mesh = get_apple_mesh(
                &self.apples,
                frame_stamp,
                self.cell_dim,
                self.prefs.draw_style,
                &self.palette,
                ctx,
                &mut self.stats,
            )?;
            draw(ctx, &apple_mesh, draw_param)?;
        }

        present(ctx)
    }
}
