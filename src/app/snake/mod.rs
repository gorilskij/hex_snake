use std::collections::VecDeque;

use ggez::{event::KeyCode, GameResult, graphics::Color};
use ggez::graphics::{DrawMode, MeshBuilder};
use mint::Point2;

use crate::app::game::{CellDim, hexagon_points};
use crate::app::hex::{Dir, Hex, HexPos, HexType::*};
use crate::app::hex::HexType;
use crate::app::palette::SnakePalette;

pub mod player_controller;
pub mod demo_controller;
pub mod snake_ai_controller;

#[derive(Eq, PartialEq)]
pub enum SnakeState {
    Living,
    Crashed,
}

pub trait SnakeController {
    fn next_dir(&mut self, snake: &SnakeRepr, other_snakes: Vec<&SnakeRepr>, apples: &[HexPos], board_dim: HexPos) -> Option<Dir>;

    fn key_pressed(&mut self, _key: KeyCode) {}
}

pub struct Snake {
    pub body: VecDeque<Hex>,
    pub palette: SnakePalette,

    pub state: SnakeState,
    pub dir: Dir,
    pub grow: usize,

    pub controller: Box<dyn SnakeController>,
    pub board_dim: HexPos,
}

pub struct SnakeRepr {
    pub body: Vec<HexPos>,
    pub dir: Dir,
}

impl<C: SnakeController + 'static> From<(HexPos, SnakePalette, Dir, usize, C, HexPos)> for Snake {
    fn from(params: (HexPos, SnakePalette, Dir, usize, C, HexPos)) -> Self {
        let (pos, palette, dir, grow, controller, game_dim) = params;

        let head = Hex {
            typ: HexType::Normal,
            pos,
            teleported: None,
        };
        let mut body = VecDeque::new();
        body.push_back(head);
        Self {
            body,
            palette,
            state: SnakeState::Living,
            dir,
            grow,
            controller: Box::new(controller),
            board_dim: game_dim,
        }
    }
}

impl Snake {
    pub fn as_repr(&self) -> SnakeRepr {
        let body = self.body
            .iter()
            .map(|Hex { pos, ..}| *pos)
            .collect::<Vec<_>>();

        SnakeRepr {
            body,
            dir: self.dir,
        }
    }

    fn get_new_head(&self) -> Hex {
        let mut new_head = Hex {
            typ: HexType::Normal,
            pos: self.body[0].pos,
            teleported: None,
        };

        // todo make O(1)
        //  at the moment this just moves the head back until the last cell that's still in the map
        //  this could be done as a single calculation
        new_head.step_and_teleport(self.dir, self.board_dim);

        new_head
    }

    pub fn advance(&mut self, other_snakes: Vec<&SnakeRepr>, apples: &[HexPos]) {
        if let Some(new_dir) = self.controller.next_dir(&self.as_repr(), other_snakes, apples, self.board_dim) {
            self.dir = new_dir;
        }

        let new_head = self.get_new_head();

        let body_last = self.body.len() - 1;
        if let HexType::Eaten(amount) = &mut self.body[body_last].typ {
            if *amount == 0 {
                self.body[body_last].typ = HexType::Normal;
            } else {
                self.grow += 1;
                *amount -= 1;
            }
        }

        if self.grow > 0 {
            self.body.insert(0, new_head);
            self.grow -= 1;
        } else {
            self.body.rotate_right(1);
            self.body[0] = new_head;
        }
    }

    pub fn draw_non_crash_points(
        &mut self,
        builder: &mut MeshBuilder,
        cell_dim: CellDim,
    ) -> GameResult {
        let len = self.body.len();
        for (i, segment) in self.body.iter().enumerate() {
            let color = match segment.typ {
                Crashed => continue,
                Normal => self.palette.segment_color.paint_segment(i, len),
                Eaten(_) => self.palette.eaten_color.paint_segment(i, len),
            };

            build_cell(builder, segment.pos, color, cell_dim)?;
        }

        Ok(())
    }

    pub fn draw_crash_point(
        &self,
        builder: &mut MeshBuilder,
        cell_dim: CellDim,
    ) -> GameResult {
        if self.body[0].typ == Crashed {
            build_cell(builder, self.body[0].pos, self.palette.crashed_color, cell_dim)?;
        }
        Ok(())
    }
}

#[inline(always)]
pub fn build_cell(
    builder: &mut MeshBuilder,
    HexPos { h, v }: HexPos,
    color: Color,
    CellDim { side, sin, cos }: CellDim,
) -> GameResult {
    let offset_x = h as f32 * (side + cos);
    let offset_y = v as f32 * 2. * sin + if h % 2 == 0 { 0. } else { sin };

    let translated_points = hexagon_points(side)
        .iter()
        .map(|Point2 { x, y }| Point2 {
            x: x + offset_x,
            y: y + offset_y,
        })
        .collect::<Vec<_>>();

    builder
        .polygon(DrawMode::fill(), &translated_points, color)
        .map(|_| ())
}