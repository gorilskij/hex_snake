use ggez::event::{EventHandler, KeyMods};
use ggez::{Context, GameError, GameResult};
use ggez::graphics::*;
use std::f32::consts::PI;
use mint::Point2;
use snake::Snake;
use ggez::input::keyboard::KeyCode;
use crate::game::snake::Dir;
use tuple::Map;
use std::thread;
use rand::prelude::*;
use crate::game::hex::Hex;
use crate::game::hex::HexType::{Apple, Crashed, Normal, Eaten};
use crate::game::hex::hex_pos::HexPos;

mod hex;
mod snake;

pub struct Game {
    running: bool,

    dim: HexPos,
    snake: Snake,
    apples: Vec<Hex>,
    
    cell_side_len: f32,

    rng: ThreadRng,
}

impl Game {
    pub fn new(horizontal: usize, vertical: usize, cell_side_len: f32) -> Self {
        let dim = HexPos { h: horizontal as isize, v: vertical as isize };
        Self {
            running: true,

            dim,
            snake: Snake::new(dim),
            apples: vec![],

            cell_side_len,

            rng: thread_rng(),
        }
    }

    // spawns apples until there are `total` apples in the game
    pub fn spawn_apples(&mut self, total: usize) {
        'outer: while self.apples.len() < total {
            let mut attempts = 0_u8;
            'apple_maker: loop {
                let apple_pos = HexPos::random_in(self.dim, &mut self.rng);
                attempts += 1;
                for s in self.snake.body.iter().chain(&self.apples) {
                    if s.pos == apple_pos {
                        // make a new apple
                        assert!(attempts < 5);
                        continue 'apple_maker;
                    }
                }

                // apple made successfully
                self.apples.push(Hex { typ: Apple, pos: apple_pos });
                continue 'outer;
            }
        }
    }
}

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        if !self.running { return Ok(()) }

        self.snake.advance();

        // check if crashed into itself
        if self.snake.crashed() {
            self.snake.body[0].typ = Crashed;
            self.running = false
        }

        // check if ate apple
        let mut delete_apple = None;
        for (i, &apple) in self.apples.iter().enumerate() {
            if self.snake.head().pos == apple.pos {
                delete_apple = Some(i)
            }
        }
        if let Some(i) = delete_apple {
            self.apples.remove(i);
            self.snake.grow(1);
            self.snake.body[0].typ = Eaten;
        }

        self.spawn_apples(50);
        thread::yield_now();
        Ok(())
    }

    // TODO: calculate how many hexagons in width and height and draw them as
    //  vertical zigzag lines, not polygons
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        // note could be fun to implement optionally printing as a square grid
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

        let apple_fill = Mesh::new_polyline(
            ctx, DrawMode::fill(),
            &hexagon_points, Color { r: 1., g: 0., b: 0., a: 1. })?;

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

        // head to tail
        for (i, segment) in self.snake.body.iter().rev().enumerate() {
            let color = match segment.typ {
                Normal => {
                    // [0.5, 1]
                    let drk = (1. - i as f32 / self.snake.body.len() as f32) / 2.;
                    Color { r: drk, b: drk, g: drk, a: 1. }
                }
                Crashed => Color { r: 1., b: 0., g: 0., a: 1. },
                Eaten => Color { r: 0., b: 0.5, g: 0.5, a: 1. },
                _ => panic!(),
            };

            let segment_fill = Mesh::new_polyline(
                ctx, DrawMode::fill(),
                &hexagon_points, color)?;

            draw_cell(segment.h as usize, segment.v as usize,
                      &segment_fill, ctx, sl, s, c)?
        }

        for apple in &self.apples {
            draw_cell(apple.h as usize, apple.v as usize,
                      &apple_fill, ctx, sl, s, c)?
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

        self.snake.set_direction_safe(new_direction);
    }
}