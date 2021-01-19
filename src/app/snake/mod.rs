use std::collections::{HashMap, VecDeque};

use ggez::{
    graphics::{Color, DrawMode, MeshBuilder},
    mint::Point2,
    Context, GameResult,
};

use crate::{
    app::{
        game::{Apple, CellDim},
        hex::{Dir, Hex, HexDim, HexPos, HexType, HexType::*},
        snake::{
            controller::{OtherSnakes, SnakeController, SnakeControllerTemplate},
            palette::{SnakePainter, SnakePaletteTemplate},
        },
        Frames,
    },
    times::Times,
};
use ggez::graphics::{spritebatch::SpriteBatch, DrawParam, Image};
use itertools::Itertools;

pub mod controller;
pub mod palette;

#[derive(Eq, PartialEq)]
pub enum SnakeState {
    Living,
    Crashed,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum SnakeType {
    PlayerSnake,
    SimulatedSnake,
    // TODO: store life information here
    CompetitorSnake { life: Option<Frames> },
    KillerSnake { life: Option<Frames> },
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum EatBehavior {
    Cut,   // cut the other snake's tail off
    Crash, // stop the game
    Die,   // disappear
}

#[derive(Clone)]
pub struct EatMechanics {
    pub eat_self: EatBehavior,
    pub eat_other: HashMap<SnakeType, EatBehavior>,
    pub default: EatBehavior,
}

pub struct SnakeBody {
    pub body: VecDeque<Hex>,
    pub dir: Dir,
    pub grow: usize,
}

pub struct Snake {
    pub snake_type: SnakeType,
    pub eat_mechanics: EatMechanics,

    pub body: SnakeBody,
    pub state: SnakeState,

    pub controller: Box<dyn SnakeController>,
    pub painter: Box<dyn SnakePainter>,
}

#[derive(Clone)]
pub struct SnakeSeed {
    pub snake_type: SnakeType,
    pub eat_mechanics: EatMechanics,
    pub palette: SnakePaletteTemplate,
    pub controller: SnakeControllerTemplate,
}

impl Snake {
    pub fn from_seed(seed: &SnakeSeed, pos: HexPos, dir: Dir, grow: usize) -> Self {
        let SnakeSeed {
            snake_type,
            eat_mechanics,
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
            snake_type,
            eat_mechanics,

            body: SnakeBody { body, dir, grow },
            state: SnakeState::Living,

            controller: controller.into(),
            painter: palette.into(),
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
}
