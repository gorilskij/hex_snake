use ggez::event::EventHandler;
use ggez::graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, MeshBuilder};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::Context;

use crate::app::game_context::GameContext;
use crate::app::keyboard_control::Controls;
use crate::app::prefs::Prefs;
use crate::app;
use crate::apple::spawn::SpawnPolicy;
use crate::basic::{CellDim, Dir, HexPoint};
use crate::error::{Error, ErrorConversion, Result};
use crate::rendering::shape::{Hexagon, Shape};
use crate::rendering::{border_mesh, grid_mesh};
use crate::snake_control::Move;

pub struct SnakeControlCreatorScreen {
    cell_dim: CellDim,
    gtx: GameContext,
    start: HexPoint,
    head: HexPoint,
    current_dir: Option<Dir>,
    path: Vec<HexPoint>,
    moves: Vec<Move>,
    controls: Controls,
    palette: app::Palette,
}

impl SnakeControlCreatorScreen {
    pub fn new(cell_dim: CellDim, controls: Controls, palette: app::Palette) -> Self {
        let board_dim = HexPoint { h: 30, v: 20 };
        let start = HexPoint { h: 15, v: 10 };
        let gtx = GameContext::new(
            board_dim,
            cell_dim,
            palette.clone(),
            Prefs::default(),
            SpawnPolicy::None,
        );
        Self {
            cell_dim,
            gtx,
            start,
            head: start,
            current_dir: None,
            path: vec![start],
            moves: vec![],
            controls,
            palette,
        }
    }

    fn press_dir(&mut self, dir: Dir) {
        if let Some(cur) = self.current_dir {
            if dir == -cur {
                eprintln!("warning: ignoring opposite direction press ({:?})", dir);
                return;
            }
            if dir == cur {
                match self.moves.last_mut() {
                    Some(Move::Wait(n)) => *n += 1,
                    _ => self.moves.push(Move::Wait(1)),
                }
            } else {
                self.moves.push(Move::Turn(dir));
                self.current_dir = Some(dir);
            }
        } else {
            self.moves.push(Move::Turn(dir));
            self.current_dir = Some(dir);
        }
        self.head += dir;
        self.path.push(self.head);
    }

    fn reset(&mut self) {
        self.head = self.start;
        self.current_dir = None;
        self.path = vec![self.start];
        self.moves.clear();
    }

    fn print_moves(&self) {
        let items: Vec<String> = self.moves.iter().map(|m| format!("{:?}", m)).collect();
        println!("vec![{}]", items.join(", "));
    }

    fn build_path_mesh(&self, ctx: &Context) -> Result<Mesh> {
        let path_len = self.path.len();
        let path_color = Color::new(0.5, 0.5, 0.5, 1.0);
        let head_color = Color::new(0.2, 0.9, 0.2, 1.0);
        let start_color = Color::new(0.9, 0.9, 0.2, 1.0);

        let mut builder = MeshBuilder::new();
        let res = try {
            for (i, &pos) in self.path.iter().enumerate() {
                let color = if i == 0 {
                    start_color
                } else if i == path_len - 1 {
                    head_color
                } else {
                    path_color
                };
                let dest = pos.to_cartesian(self.cell_dim);
                let points = Hexagon::new(self.cell_dim).translate(dest);
                builder.polygon(DrawMode::fill(), &points, color)?;
            }
            Mesh::from_data(ctx, builder.build())
        };
        res.map_err(Error::from).with_trace_step("SnakeControlCreatorScreen::build_path_mesh")
    }
}

impl EventHandler<Error> for SnakeControlCreatorScreen {
    fn update(&mut self, _ctx: &mut Context) -> Result {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result {
        let mut canvas = Canvas::from_frame(ctx, self.palette.background_color);

        let grid = grid_mesh(&self.gtx, ctx)?;
        canvas.draw(&grid, DrawParam::default());

        let border = border_mesh(&self.gtx, ctx)?;
        canvas.draw(&border, DrawParam::default());

        let path_mesh = self.build_path_mesh(ctx)?;
        canvas.draw(&path_mesh, DrawParam::default());

        canvas.finish(ctx).map_err(Error::from).with_trace_step("SnakeControlCreatorScreen::draw")
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> Result {
        use Dir::*;
        let controls = self.controls;

        match input.keycode {
            Some(k) if k == controls.u  => self.press_dir(U),
            Some(k) if k == controls.d  => self.press_dir(D),
            Some(k) if k == controls.ul => self.press_dir(Ul),
            Some(k) if k == controls.ur => self.press_dir(Ur),
            Some(k) if k == controls.dl => self.press_dir(Dl),
            Some(k) if k == controls.dr => self.press_dir(Dr),
            Some(KeyCode::Space)  => self.print_moves(),
            Some(KeyCode::Escape) => self.reset(),
            _ => {}
        }

        Ok(())
    }
}
