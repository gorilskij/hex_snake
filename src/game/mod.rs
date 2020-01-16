use ggez::event::{EventHandler, KeyMods};
use ggez::{Context, GameError};
use ggez::graphics::{clear, WHITE, present, MeshBuilder, DrawMode, FillOptions, BLACK, draw, DrawParam, StrokeOptions, Color};
use std::f32::consts::PI;
use mint::Point2;
use crate::{HEIGHT, WIDTH};
use hex_grid_point::HexGridPoint;
use snake::Snake;
use ggez::input::keyboard::KeyCode;
use crate::game::snake::Dir;
use std::time::Duration;
use tuple::Map;
use std::thread;

mod hex_grid_point;
mod snake;

// hexagonal cell side length
//const L: f32 = 10.;

pub struct Game {
    shape: HexGridPoint,
    snake: Snake,
    
    cell_side_len: f32,
}

impl Game {
    pub fn new(horizontal: usize, vertical: usize, cell_side_len: f32) -> Self {
        Self {
            shape: HexGridPoint { h: horizontal, v: vertical },
            snake: Snake::new(horizontal, vertical),
            cell_side_len,
        }
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.snake.advance();
        thread::sleep(Duration::from_millis(500));
        Ok(())
    }

    // TODO: calculate how many hexagons in width and height and draw them as
    //  vertical zigzag lines, not polygons
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        clear(ctx, WHITE);

        let mut builder = MeshBuilder::new();

        for hor in 0..self.shape.h {
            for ver in 0..self.shape.v {
                let a = 1. / 3. * PI; // 120deg
                let sl = self.cell_side_len;
                let (s, c) = a.sin_cos().map(|x| x * sl);

                // with 0, 0 the board is touching top-left (nothing hidden)
                let (dx, dy) = (0., 0.);

                let hexagon = [
                    (c, 0.),
                    (sl + c, 0.),
                    (sl + 2. * c, s),
                    (sl + c, 2. * s),
                    (c, 2. * s),
                    (0., s),
                    (c, 0.),
                ].iter()
                    .map(|&(x, y)| Point2 {
                        x: dx + x + hor as f32 * (sl + c),
                        y: dy + y + ver as f32 * 2. * s + if hor % 2 == 0 { 0. } else { s },
                    })
                    .collect::<Vec<_>>();

                if self.snake.is_at(HexGridPoint { h: hor, v: ver }) {
                    builder.polyline(
                        DrawMode::Fill(FillOptions::DEFAULT),
                        &hexagon,
                        Color::new(0., 0.8, 0., 1.),
                    )?;
                } else {
                    builder.polyline(
                        DrawMode::Stroke(StrokeOptions::DEFAULT),
                        &hexagon,
                        BLACK,
                    )?;
                }
            }
        }

        let mesh = builder.build(ctx)?;
        draw(ctx, &mesh, DrawParam::new())?;

        present(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, _: bool) {
        let new_direction = match key {
            KeyCode::H => Dir::LeftUp,
            KeyCode::T => Dir::Up,
            KeyCode::N => Dir::RightUp,
            KeyCode::M => Dir::LeftDown,
            KeyCode::W => Dir::Down,
            KeyCode::V => Dir::RightDown,
            _ => return,
        };
        self.snake.set_direction(new_direction);
    }
}