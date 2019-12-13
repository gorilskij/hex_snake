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

mod hex_grid_point;
mod snake;

// hexagonal cell side length
const L: f32 = 10.;

pub struct Game {
    shape: HexGridPoint,
    snake: Snake,
}

impl Game {
    pub fn new(horizontal: usize, vertical: usize) -> Self {
        Self {
            shape: HexGridPoint { h: horizontal, v: vertical },
            snake: Snake::new(horizontal / 2, vertical / 2),
        }
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.snake.advance();
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
                let (sin, cos) = (a.sin(), a.cos());
                let hexagon = [
                    (0., 0.),
                    (L, 0.),
                    (L + L * cos, L * sin),
                    (L, 2. * L * sin),
                    (0., 2. * L * sin),
                    (-L * cos, L * sin),
                    (0., 0.),
                ].iter()
                    .map(|&(x, y)| Point2 {
                        x: x + hor as f32 * (L + L * cos),
                        y: y + ver as f32 * 2. * L * sin + if hor % 2 == 0 { 0. } else { L * sin },
                    })
                    .collect::<Vec<_>>();

                let snake_here = self.snake.is_at(HexGridPoint { h: hor, v: ver });

                if snake_here {
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