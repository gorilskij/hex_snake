use Dir::*;
use std::ops::Neg;
use super::hex::{Hex, HexPos, HexType::*};
use ggez::{Context, GameResult};
use ggez::graphics::{Color, Mesh, DrawMode};
use mint::Point2;
use crate::game::ctrl::Ctrl;
use crate::game::theme::Palette;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Dir { U, D, UL, UR, DL, DR }

impl Neg for Dir {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            U => D, D => U, UL => DR, UR => DL, DL => UR, DR => UL,
        }
    }
}


#[derive(Eq, PartialEq)]
pub enum SnakeState { Living, Crashed }

pub struct Snake {
    pub body: Vec<Hex>,
    growing: usize,
    dir: Dir,
    pub game_dim: HexPos, // cached value

    pub state: SnakeState,

    pub ctrl: Ctrl,
}

impl Snake {
    pub fn new(dim: HexPos, offset: HexPos, ctrl: Ctrl) -> Self {
        let center = Hex { typ: Normal, pos: dim / 2 + offset };
        Self {
            body: vec![center],
            growing: 15,
            dir: Dir::U,
            game_dim: dim,

            state: SnakeState::Living,

            ctrl,
        }
    }

    pub fn grow(&mut self, amount: usize) {
        self.growing += amount
    }

    pub fn advance(&mut self) {
        let mut new_head = Hex {
            typ: Normal,
            pos: self.body[0].pos,
        };

        // todo make O(1)
        new_head.translate(self.dir, 1);
        if !new_head.is_in(self.game_dim) {
            // step back
            new_head.translate(self.dir, -1);

            while new_head.is_in(self.game_dim) {
                new_head.translate(self.dir, -1);
            }
            new_head.translate(self.dir, 1);
        }

        if self.growing > 0 {
            self.body.insert(0, new_head);
            self.growing -= 1;
        } else {
            self.body.rotate_right(1);
            self.body[0] = new_head;
        }
    }

    // ignore opposite direction
    pub fn set_direction_safe(&mut self, new_dir: Dir) {
        if self.dir != -new_dir { self.dir = new_dir }
    }

    pub fn draw_non_crash_points(
        &self,
        ctx: &mut Context,
        hexagon_points: &[Point2<f32>],
        draw_cell: fn(usize, usize, &Mesh, &mut Context, f32, f32, f32) -> GameResult,
        sl: f32,
        s: f32,
        c: f32,
        palette: &Palette,
    ) -> GameResult {
        let head = palette.snake_head_color;
        let tail = palette.snake_tail_color;

        // head to tail
        for (i, segment) in self.body.iter().enumerate() {
            if segment.typ == Crashed {
                continue
            }

            let color = match segment.typ {
                Normal => {
                    // darkness of the color, range: [0.5, 1]
                    // let darkness = (1. - i as f32 / self.body.len() as f32) / 2.;
                    let head_color_ratio = 1. - i as f32 / (self.body.len() - 1) as f32;
                    let tail_color_ratio = 1. - head_color_ratio;
                    Color {
                        r: head_color_ratio * head.r + tail_color_ratio * tail.r,
                        g: head_color_ratio * head.g + tail_color_ratio * tail.g,
                        b: head_color_ratio * head.b + tail_color_ratio * tail.b,
                        a: 1.,
                    }
                }
                Eaten => Color { r: 0., b: 0.5, g: 1., a: 1. }, // todo darkening for these too
                _ => panic!(),
            };

            let segment_fill = Mesh::new_polyline(
                ctx, DrawMode::fill(),
                hexagon_points, color)?;

            draw_cell(segment.h as usize, segment.v as usize,
                      &segment_fill, ctx, sl, s, c)?
        }

        Ok(())
    }

    pub fn draw_crash_point(
        &self,
        ctx: &mut Context,
        hexagon_points: &[Point2<f32>],
        draw_cell: fn(usize, usize, &Mesh, &mut Context, f32, f32, f32) -> GameResult,
        sl: f32,
        s: f32,
        c: f32,
        palette: &Palette,
    ) -> GameResult {
        let color = Color { r: 1., b: 0.5, g: 0., a: 1. };
        if self.body[0].typ == Crashed {
            let segment_fill = Mesh::new_polyline(
                ctx, DrawMode::fill(),
                hexagon_points, color)?;
            draw_cell(self.body[0].h as usize, self.body[0].v as usize,
                      &segment_fill, ctx, sl, s, c)?
        }
        Ok(())
    }
}