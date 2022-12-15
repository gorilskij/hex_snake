use crate::basic::{Food, HexPoint};
use crate::snake;

#[macro_use]
pub mod spawn;

#[derive(Debug)]
pub enum Type {
    Food(Food),
    SpawnSnake(Box<snake::Builder>),
    SpawnRain,
}

impl Type {
    pub fn is_animated(&self) -> bool {
        match self {
            Type::Food(_) => false,
            Type::SpawnSnake(_) => true,
            Type::SpawnRain => true,
        }
    }
}

pub struct Apple {
    pub pos: HexPoint,
    pub apple_type: Type,
}
