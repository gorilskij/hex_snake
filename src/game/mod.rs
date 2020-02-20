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
use hex::hex_pos::IsEven;
use crate::game::effect::Effect;
use std::time::Duration;

mod hex;
mod snake;
mod effect;

struct Ctrl {
    u: KeyCode,
    d: KeyCode,
    ul: KeyCode,
    ur: KeyCode,
    dl: KeyCode,
    dr: KeyCode,
}

pub struct Game {
    running: bool,

    dim: HexPos,
    snakes: Vec<(Ctrl, Snake)>,
    apples: Vec<Hex>,
    
    cell_side_len: f32,

    rng: ThreadRng,
    grid_mesh: Option<Mesh>,
    effect: Option<Effect>,
}

impl Game {
    pub fn new(horizontal: usize, vertical: usize, cell_side_len: f32) -> Self {
        let dim = HexPos { h: horizontal as isize, v: vertical as isize };

        let ctrl_right_hand_dvorak = Ctrl {
            u: KeyCode::T,
            d: KeyCode::W,
            ul: KeyCode::H,
            ur: KeyCode::N,
            dl: KeyCode::M,
            dr: KeyCode::V,
        };

        let ctrl_left_hand_dvorak = Ctrl {
            u: KeyCode::E,
            d: KeyCode::Q,
            ul: KeyCode::O,
            ur: KeyCode::U,
            dl: KeyCode::Semicolon,
            dr: KeyCode::J,
        };

        Self {
            running: true,

            dim,
            snakes: vec![
                (ctrl_left_hand_dvorak, Snake::new(dim, HexPos { h: -2, v: 0 })),
                (ctrl_right_hand_dvorak, Snake::new(dim, HexPos { h: 2, v: 0 })),
            ],
            apples: vec![],

            cell_side_len,

            rng: thread_rng(),
            grid_mesh: None,
            effect: None,
        }
    }

    // spawns apples until there are `total` apples in the game
    pub fn spawn_apples(&mut self, total: usize) {
        'outer: while self.apples.len() < total {
            let mut attempts = 0_u8;
            'apple_maker: loop {
                let apple_pos = HexPos::random_in(self.dim, &mut self.rng);
                attempts += 1;
                for s in self
                    .snakes
                    .iter()
                    .map(|(_, s)| s.body.iter())
                    .flatten()
                    .chain(&self.apples)
                {
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

        thread::sleep(Duration::from_millis(100));

        for (_, snake) in &mut self.snakes {
            snake.advance()
        }

        // check if crashed
        let mut crashed_snake_indices = vec![];
        'outer: for (i, (_, snake)) in self.snakes.iter().enumerate() {
            for (_, other) in &self.snakes {
                for segment in &other.body[1..] {
                    if snake.body[0].pos == segment.pos {
//                        snake.body[0].typ = Crashed;
                        crashed_snake_indices.push(i);
                        self.running = false;
                        continue 'outer;
                    }
                }
            }
        }
        for i in crashed_snake_indices {
            self.snakes[i].1.body[0].typ = Crashed
        }

        // check if ate apple
        let mut eaten_apples = vec![];
        for (a, &apple) in self.apples.iter().enumerate() {
            for (s, (_, snake)) in self.snakes.iter().enumerate() {
                if snake.body[0].pos == apple.pos {
                    eaten_apples.push((a, s))
                }
            }
        }
        // reverse index order so removal doesn't affect later apples
        eaten_apples.sort_by(|(a1, _), (a2, _)| a1.cmp(a2));
        for &(a, s) in eaten_apples.iter().rev() {
            self.apples.remove(a);
            self.snakes[s].1.grow(1);
            self.snakes[s].1.body[0].typ = Eaten;

            // apply effect, might be too much with multiple snakes
            // todo apply effect per-snake
//            self.effect = Some(Effect::SmallHex {
//                min_scale: 0.2,
//                iterations: 10,
//                passed: 0,
//            });
        }

        self.spawn_apples(50);

        thread::yield_now();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        // note could be fun to implement optionally printing as a square grid
        // TODO: reimplement optional pushing to left-top hiding part of the hexagon
        // with 0, 0 the board is touching top-left (nothing hidden)

        clear(ctx, WHITE);

        // general useful lengths
        let a = 1. / 3. * PI; // 120deg
        let sl = self.cell_side_len;
        let (s, c) = a.sin_cos().map(|x| x * sl);

        // generate grid mesh first time, later reuse
        if let None = self.grid_mesh {
            let mut wall_points_a = vec![];
            let mut wall_points_b = vec![];

            let mut between_a_b = vec![];
            let mut between_b_a = vec![];

            for v in 0 .. self.dim.v {
                let dv = v as f32 * 2. * s;

                wall_points_a.push(Point2 { x: c, y: dv });
                wall_points_a.push(Point2 { x: 0., y: dv + s });

                wall_points_b.push(Point2 { x: c + sl, y: dv });
                wall_points_b.push(Point2 { x: 2. * c + sl, y: dv + s });

                between_a_b.push(Point2 { x: c, y: dv });
                between_a_b.push(Point2 { x: c + sl, y: dv });

                between_b_a.push(Point2 { x: 2. * c + sl, y: dv + s });
                between_b_a.push(Point2 { x: 2. * c + 2. * sl, y: dv + s });
            }
            {
                let dv = self.dim.v as f32 * 2. * s;
                wall_points_a.push(Point2 { x: c, y: dv });
                wall_points_b.push(Point2 { x: c + sl, y: dv });
            }

            let mut builder = MeshBuilder::new();

            let draw_mode = DrawMode::stroke(2.);
            for h in 0 .. (self.dim.h + 1) / 2 {
                builder.polyline(draw_mode, &wall_points_a, BLACK)?;
                builder.polyline(draw_mode, &wall_points_b, BLACK)?;

                let dh = h as f32 * (2. * sl + 2. * c);

                for v in 0 .. self.dim.v {
                    let dv = v as f32 * 2. * s;
                    builder.line(&[
                        Point2 { x: c + dh, y: dv },
                        Point2 { x: c + sl + dh, y: dv }
                    ], 2., BLACK)?;

                    builder.line(&[
                        Point2 { x: 2. * c + sl + dh, y: s + dv },
                        Point2 { x: 2. * c + 2. * sl + dh, y: s + dv },
                    ], 2., BLACK)?;
                }
                {
                    let dv = self.dim.v as f32 * 2. * s;
                    builder.line(&[
                        Point2 { x: c + dh, y: dv },
                        Point2 { x: c + sl + dh, y: dv }
                    ], 2., BLACK)?;
                }

                for (a, b) in wall_points_a
                    .iter_mut()
                    .zip(&mut wall_points_b) {
                    a.x += 2. * sl + 2. * c;
                    b.x += 2. * sl + 2. * c;
                }
            }
            if self.dim.h.is_even() {
                builder.polyline(draw_mode, &wall_points_a[..wall_points_a.len() - 1],
                                 BLACK)?;
            }

            self.grid_mesh = Some(builder.build(ctx)?);
        }
        draw(ctx, self.grid_mesh.as_ref().unwrap(), DrawParam::default())?;

        let mut hexagon_points = [
            Point2 { x: c, y: 0. },
            Point2 { x: sl + c, y: 0. },
            Point2 { x: sl + 2. * c, y: s },
            Point2 { x: sl + c, y: 2. * s },
            Point2 { x: c, y: 2. * s },
            Point2 { x: 0., y: s },
            Point2 { x: c, y: 0. },
        ];

        if let Some(Effect::SmallHex {
                        min_scale,
                        iterations,
                        passed
                    }) = self.effect {
            let scale_factor = if passed < iterations / 2 {
                let fraction = passed as f32 / (iterations as f32 / 2.);
                1. - fraction * (1. - min_scale)
            } else {
                let fraction = (passed - iterations / 2) as f32 / (iterations - iterations / 2) as f32;
                1. - (1. - fraction) * (1. - min_scale)
            };

            // scale down and reposition in the middle of the hexagon
            for pt in &mut hexagon_points {
                pt.x *= scale_factor;
                pt.y *= scale_factor;
                // formula is (dimension / 2) * (1 - scale factor) [simplified]
                pt.x += (c + sl / 2.) * (1. - scale_factor);
                pt.y += s * (1. - scale_factor); // actually 2 * s / 2
            }

            if passed == iterations {
                self.effect = None;
            } else {
                // always succeeds, only used to unwrap
                if let Some(Effect::SmallHex { passed, .. }) = &mut self.effect {
                    *passed += 1
                }
            }
        }
        
        let apple_fill = Mesh::new_polyline(
            ctx, DrawMode::fill(),
            &hexagon_points, Color { r: 1., g: 0., b: 0., a: 1. })?;


        // todo supersede
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

        // draw snakes, crashed snakes on top (last)
        for (_, snake) in self
            .snakes
            .iter()
            .filter(|(_, s)| s.body[0].typ != Crashed)
        {
            snake.draw(ctx, &hexagon_points, draw_cell, sl, s, c)?
        }
        for (_, snake) in self
            .snakes
            .iter()
            .filter(|(_, s)| s.body[0].typ == Crashed)
        {
            snake.draw(ctx, &hexagon_points, draw_cell, sl, s, c)?
        }

        for apple in &self.apples {
            draw_cell(apple.h as usize, apple.v as usize,
                      &apple_fill, ctx, sl, s, c)?
        }

//        println!("draw: {}ms", start.elapsed().as_millis());

        thread::yield_now();
        present(ctx)
    }

    fn key_down_event(&mut self, _ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
        use Dir::*;
        for (ctrl, snake) in &mut self.snakes {
            let new_dir = match key {
                k if k == ctrl.u => Some(U),
                k if k == ctrl.d => Some(D),
                k if k == ctrl.ul => Some(UL),
                k if k == ctrl.ur => Some(UR),
                k if k == ctrl.dl => Some(DL),
                k if k == ctrl.dr => Some(DR),
                _ => None,
            };

            if let Some(d) = new_dir {
                snake.set_direction_safe(d)
            }
        }
    }
}