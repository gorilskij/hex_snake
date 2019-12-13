use ggez::event::EventHandler;
use ggez::{Context, GameError};
use ggez::graphics::{clear, WHITE, present, MeshBuilder, DrawMode, FillOptions, BLACK, draw, DrawParam, StrokeOptions, Color};
use std::f32::consts::PI;
use mint::Point2;
use crate::{HEIGHT, WIDTH};

// hexagonal cell side length
const L: f32 = 10.;

struct HexGridPoint {
    h: usize,
    v: usize,
}

pub struct Game {
    shape: HexGridPoint,
    snake: Vec<HexGridPoint>,
}

impl Game {
    pub(crate) fn new(horizontal: usize, vertical: usize) -> Self {
        Self {
            shape: HexGridPoint { h: horizontal, v: vertical },
            snake: vec![HexGridPoint { h: 20, v: 20 }],
        }
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
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

                let snake_here = self.snake.iter()
                    .any(|&HexGridPoint { h, v }| h == hor && v == ver);

                if snake_here {
                    println!("{:?}", hexagon);
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
}