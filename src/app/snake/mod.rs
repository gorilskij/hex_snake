use crate::app::hex::{Hex, HexPos, HexType::*, Dir, Dir::*};
use ggez::{event::KeyCode, graphics::Color, GameResult};
use std::{collections::VecDeque, ops::Neg};
use crate::app::hex::HexType;
use crate::app::ctrl::Controls;
use crate::app::palette::{Palette, SnakePalette};
use ggez::graphics::{MeshBuilder, DrawMode};
use mint::Point2;
use crate::app::game::{CellDim, HEXAGON_POINTS};

pub mod player_snake;
pub mod sim_snake;

#[derive(Eq, PartialEq)]
pub enum SnakeState {
    Living,
    Crashed,
}

pub trait SnakeController {
    fn next_dir(&mut self, dir: Dir) -> Dir;

    fn key_pressed(&mut self, _key: KeyCode) {}
}

pub struct Snake<C: SnakeController> {
    pub body: Vec<Hex>,
    pub palette: SnakePalette,

    pub state: SnakeState,
    pub dir: Dir,
    pub grow: usize,

    pub controller: C,
    pub game_dim: HexPos,
}

impl<C: SnakeController> From<(HexPos, SnakePalette, Dir, usize, C, HexPos)> for Snake<C> {
    fn from((pos, palette, dir, grow, controller, game_dim):
            (HexPos, SnakePalette, Dir, usize, C, HexPos)) -> Self {
        Self {
            body: vec![Hex {
                typ: HexType::Normal,
                pos,
                teleported: None,
            }],
            palette,
            state: SnakeState::Living,
            dir,
            grow,
            controller,
            game_dim,
        }
    }
}

impl<C: SnakeController> Snake<C> {
    fn get_new_head(&self) -> Hex {
        let mut new_head = Hex {
            typ: HexType::Normal,
            pos: self.body[0].pos,
            teleported: None,
        };

        // todo make O(1)
        //  at the moment this just moves the head back until the last cell that's still in the map
        //  this could be done as a single calculation
        new_head.translate(self.dir, 1);
        if !new_head.is_in(self.game_dim) {
            // find reappearance point
            new_head.translate(self.dir, -1);
            while new_head.is_in(self.game_dim) {
                new_head.translate(self.dir, -1);
            }
            new_head.translate(self.dir, 1);
        }

        new_head
    }

    pub fn advance(&mut self) {
        self.dir = self.controller.next_dir(self.dir);

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
        &self,
        draw_cell: &mut impl FnMut(usize, usize, Color, Option<Dir>) -> GameResult,
    ) -> GameResult {
        let len = self.body.len();
        for (i, segment) in self.body.iter().enumerate() {
            let color = match segment.typ {
                Crashed => continue,
                Normal => self.palette.segment_color.as_ref()(i, len),
                Eaten(_) => self.palette.eaten_color.as_ref()(i, len),
            };

            match segment.teleported {
                None => draw_cell(segment.h as usize, segment.v as usize, color, None)?,
                Some(dir) => {
                    draw_cell(segment.h as usize, segment.v as usize, color, Some(-dir))?;
                    draw_cell(segment.h as usize, segment.v as usize, self.palette.portal_color, Some(dir))?;
                },
            }
        }

        Ok(())
    }

    pub fn draw_crash_point(
        &self,
        draw_cell: &mut impl FnMut(usize, usize, Color, Option<Dir>) -> GameResult,
    ) -> GameResult {
        if self.body[0].typ == Crashed {
            draw_cell(
                self.body[0].h as usize,
                self.body[0].v as usize,
                self.palette.crashed_color,
                None,
            )?
        }
        Ok(())
    }
}

pub fn draw_cell(builder: &mut MeshBuilder, h: usize, v: usize, c: Color, dir: Option<Dir>) -> GameResult {
    let cell_dim = CellDim::from(10.);

    let offset_x = h as f32 * (cell_dim.side + cell_dim.cos);
    let offset_y =
        v as f32 * 2. * cell_dim.sin + if h % 2 == 0 { 0. } else { cell_dim.sin };

    use Dir::*;
    let points: &[_] = match dir {
        None => &HEXAGON_POINTS.full,
        Some(U) => &HEXAGON_POINTS.u,
        Some(D) => &HEXAGON_POINTS.d,
        Some(UL) => &HEXAGON_POINTS.ul,
        Some(UR) => &HEXAGON_POINTS.ur,
        Some(DL) => &HEXAGON_POINTS.dl,
        Some(DR) => &HEXAGON_POINTS.dr,
    };

    let translated_points = points
        .iter()
        .map(|Point2 { x, y }| Point2 {
            x: x + offset_x,
            y: y + offset_y,
        })
        .collect::<Vec<_>>();
    builder
        .polyline(DrawMode::fill(), &translated_points, c)
        .map(|_| ())
}