use std::collections::VecDeque;

use ggez::{
    graphics::{Color, DrawMode, MeshBuilder},
    GameResult,
};
use mint::Point2;

use crate::app::{
    game::{hexagon_points, Apple, CellDim},
    hex::{Dir, Hex, HexDim, HexPos, HexType, HexType::*},
    snake::{
        controller::{OtherSnakes, SnakeController, SnakeControllerTemplate},
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

#[derive(Clone, Eq, PartialEq)]
pub enum SnakeType {
    PlayerSnake,
    SimulatedSnake,
    // TODO: store life information here
    CompetitorSnake,
    KillerSnake,
}

pub struct SnakeBody {
    pub body: VecDeque<Hex>,
    pub dir: Dir,
    pub grow: usize,
}

pub struct Snake {
    pub snake_type: SnakeType,
    pub body: SnakeBody,

    pub state: SnakeState,

    pub controller: Box<dyn SnakeController>,
    pub painter: Box<dyn SnakePainter>,

    pub life: Option<u32>,
}

#[derive(Clone)]
pub struct SnakeSeed {
    pub snake_type: SnakeType,
    pub palette: SnakePaletteTemplate,
    pub controller: SnakeControllerTemplate,
    pub life: Option<u32>,
}

impl Snake {
    pub fn from_seed(seed: &SnakeSeed, pos: HexPos, dir: Dir, grow: usize) -> Self {
        let SnakeSeed {
            snake_type,
            palette,
            controller,
            life,
        } = (*seed).clone();

        let head = Hex {
            typ: HexType::Normal,
            pos,
            teleported: None,
        };

        let mut body = VecDeque::new();
        body.push_back(head);

        Self {
            snake_type,
            body: SnakeBody { body, dir, grow },

            state: SnakeState::Living,

            controller: controller.into(),
            painter: palette.into(),

            life,
        }
    }

    pub fn len(&self) -> usize {
        self.body.body.len()
    }

    pub fn dir(&self) -> Dir {
        self.body.dir
    }

    pub fn head(&self) -> &Hex {
        &self.body.body[0]
    }

    pub fn advance(&mut self, other_snakes: OtherSnakes, apples: &[Apple], board_dim: HexDim) {
        // determine new direction for snake
        if let Some(new_dir) = self
            .controller
            .next_dir(&self.body, other_snakes, apples, board_dim)
        {
            self.body.dir = new_dir
        }

        // create new head for snake
        let mut new_head = Hex {
            typ: HexType::Normal,
            pos: self.head().pos,
            teleported: None,
        };

        new_head.step_and_teleport(self.dir(), board_dim);

        let last_idx = self.len() - 1;
        if let HexType::Eaten(amount) = &mut self.body.body[last_idx].typ {
            if *amount == 0 {
                self.body.body[last_idx].typ = HexType::Normal;
            } else {
                self.body.grow += 1;
                *amount -= 1;
            }
        }

        if self.body.grow > 0 {
            self.body.body.push_front(new_head);
            self.body.grow -= 1;
        } else {
            self.body.body.rotate_right(1);
            self.body.body[0] = new_head;
        }
    }

    pub fn draw_non_crash_points(
        &mut self,
        builder: &mut MeshBuilder,
        cell_dim: CellDim,
    ) -> GameResult {
        let len = self.len();
        for (seg_idx, hex) in self.body.body.iter().enumerate() {
            let color = self.painter.paint_segment(seg_idx, len, hex);
            build_cell(builder, hex.pos, color, cell_dim)?;
        }

        Ok(())
    }

    pub fn draw_crash_point(&mut self, builder: &mut MeshBuilder, cell_dim: CellDim) -> GameResult {
        if self.head().typ == Crashed {
            build_cell(
                builder,
                self.head().pos,
                self.painter
                    .paint_segment(0, self.len(), &self.body.body[0]),
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
