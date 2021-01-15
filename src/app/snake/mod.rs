use std::collections::VecDeque;

use ggez::{
    graphics::{Color, DrawMode, MeshBuilder},
    GameResult,
};
use mint::Point2;

use crate::app::{
    game::{hexagon_points, CellDim},
    hex::{Dir, Hex, HexPos, HexType, HexType::*},
    snake::{
        controller::{SnakeController, SnakeControllerTemplate},
        palette::{SnakePainter, SnakePaletteTemplate},
    },
};

pub mod controller;
pub mod palette;

#[derive(Eq, PartialEq)]
pub enum SnakeState {
    Living,
    Crashed,
}

pub struct Snake {
    pub body: VecDeque<Hex>,
    pub painter: Box<dyn SnakePainter>,

    pub state: SnakeState,
    pub dir: Dir,
    pub grow: usize,

    pub controller: Box<dyn SnakeController>,
    pub board_dim: HexPos,
}

#[derive(Clone)]
pub struct SnakeSeed {
    pub palette: SnakePaletteTemplate,
    pub controller: SnakeControllerTemplate,
}

impl Snake {
    pub fn from_seed(
        seed: &SnakeSeed,
        pos: HexPos,
        dir: Dir,
        grow: usize,
        board_dim: HexPos,
    ) -> Self {
        let SnakeSeed {
            palette,
            controller,
        } = (*seed).clone();

        let head = Hex {
            typ: HexType::Normal,
            pos,
            teleported: None,
        };

        let mut body = VecDeque::new();
        body.push_back(head);

        Self {
            body,
            painter: palette.into(),
            state: SnakeState::Living,
            dir,
            grow,
            controller: controller.into(),
            board_dim,
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

    pub fn advance(&mut self, other_bodies: &[&VecDeque<Hex>], apples: &[HexPos]) {
        if let Some(new_dir) =
            self.controller
                .next_dir(&self.body, self.dir, other_bodies, apples, self.board_dim)
        {
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
        for (seg_idx, hex) in self.body.iter().enumerate() {
            let color = self.painter.paint_segment(seg_idx, len, hex);
            build_cell(builder, hex.pos, color, cell_dim)?;
        }

        Ok(())
    }

    pub fn draw_crash_point(&mut self, builder: &mut MeshBuilder, cell_dim: CellDim) -> GameResult {
        if self.body[0].typ == Crashed {
            build_cell(
                builder,
                self.body[0].pos,
                self.painter
                    .paint_segment(0, self.body.len(), &self.body[0]),
                cell_dim,
            )?;
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
