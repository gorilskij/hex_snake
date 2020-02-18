use ggez::event::{EventHandler, KeyMods};
use ggez::{Context, GameError, GameResult};
use ggez::graphics::{clear, WHITE, present, DrawMode, BLACK, draw, DrawParam, StrokeOptions, Mesh, Drawable};
use std::f32::consts::PI;
use mint::Point2;
use hex_grid_point::HexGridPoint;
use snake::Snake;
use ggez::input::keyboard::KeyCode;
use crate::game::snake::Dir;
use tuple::Map;
use std::thread;
use std::time::Duration;

mod hex_grid_point;
mod snake;

pub struct Game {
    dim: HexGridPoint,
    snake: Snake,
    
    cell_side_len: f32,
}

impl Game {
    pub fn new(horizontal: usize, vertical: usize, cell_side_len: f32) -> Self {
        let dim = HexGridPoint { h: horizontal as isize, v: vertical as isize };
        Self {
            dim,
            snake: Snake::new(dim),
            cell_side_len,
        }
    }
}

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        self.snake.advance();
        thread::yield_now();
        Ok(())
    }

    // TODO: calculate how many hexagons in width and height and draw them as
    //  vertical zigzag lines, not polygons
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        // TODO: reimplement optional pushing to left-top hiding part of the hexagon
        // with 0, 0 the board is touching top-left (nothing hidden)
//        let (dx, dy) = (0., 0.);

        let a = 1. / 3. * PI; // 120deg
        let sl = self.cell_side_len;
        let (s, c) = a.sin_cos().map(|x| x * sl);

        let hexagon_points = [
            (c, 0.),
            (sl + c, 0.),
            (sl + 2. * c, s),
            (sl + c, 2. * s),
            (c, 2. * s),
            (0., s),
            (c, 0.),
        ].iter()
            .map(|&(x, y)| Point2 { x, y })
            .collect::<Vec<_>>();

        let hexagon_stroke = Mesh::new_polyline(
            ctx, DrawMode::Stroke(StrokeOptions::default()),
            &hexagon_points, BLACK)?;

        let hexagon_fill = Mesh::new_polyline(
            ctx, DrawMode::fill(),
            &hexagon_points, BLACK)?;

        clear(ctx, WHITE);

        #[inline(always)]
        fn draw_cell<D: Drawable>(
            h: usize,
            v: usize,
            drawable: &D,
            ctx: &mut Context,
            sl: f32,
            s: f32,
            c: f32,
        ) -> GameResult<()> {
            let point = Point2 {
                x: h as f32 * (sl + c),
                y: v as f32 * 2. * s + if h % 2 == 0 { 0. } else { s },
            };

            draw(ctx, drawable,
                 DrawParam::from((point, 0.0, WHITE)))
        }

        for h in 0..self.dim.h as usize {
            for v in 0..self.dim.v as usize {
                draw_cell(h, v, &hexagon_stroke, ctx, sl, s, c)?
            }
        }

        for segment in &self.snake.body {
            draw_cell(segment.h as usize, segment.v as usize, &hexagon_fill, ctx, sl, s, c)?
        }

        present(ctx)
    }

    fn key_down_event(&mut self, _ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
        use KeyCode::*;
        let new_direction = match key {
            H => Dir::UL,
            T => Dir::U,
            N => Dir::UR,
            M => Dir::DL,
            W => Dir::D,
            V => Dir::DR,
            _ => return,
        };

        self.snake.set_direction(new_direction);
    }
}