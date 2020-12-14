use crate::app::hex::{Hex, HexPos, HexType::*, Dir, Dir::*};
use ggez::{event::KeyCode, graphics::Color, GameResult};
use std::{collections::VecDeque, ops::Neg};
use crate::app::hex::HexType;
use crate::app::ctrl::Controls;
use crate::app::palette::{Palette, SnakePalette};

pub mod player_snake;
pub mod sim_snake;

pub trait Snake {
    fn body(&self) -> &Vec<Hex>;

    fn palette(&self) -> &SnakePalette;

    fn advance(&mut self);

    fn draw_non_crash_points(
        &self,
        draw_cell: &mut impl FnMut(usize, usize, Color, Option<Dir>) -> GameResult,
    ) -> GameResult {
        let palette = self.palette();

        let len = self.body().len();
        for (i, segment) in self.body().iter().enumerate() {
            let color = match segment.typ {
                Crashed => continue,
                Normal => palette(i, len),
                Eaten(_) => palette.eaten_color.as_ref()(i, len),
            };

            match segment.teleported {
                None => draw_cell(segment.h as usize, segment.v as usize, color, None)?,
                Some(dir) => {
                    draw_cell(segment.h as usize, segment.v as usize, color, Some(-dir))?;
                    draw_cell(segment.h as usize, segment.v as usize, palette.portal_color, Some(dir))?;
                },
            }
        }

        Ok(())
    }

    fn draw_crash_point(
        &self,
        draw_cell: &mut impl FnMut(usize, usize, Color, Option<Dir>) -> GameResult,
    ) -> GameResult {
        if self.body()[0].typ == Crashed {
            draw_cell(
                self.body()[0].h as usize,
                self.body()[0].v as usize,
                self.palette().crashed_color,
                None,
            )?
        }
        Ok(())
    }
}

#[derive(Eq, PartialEq)]
pub enum SnakeState {
    Living,
    Crashed,
}