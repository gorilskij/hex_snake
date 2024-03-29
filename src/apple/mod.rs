use crate::basic::{Food, HexPoint};
use crate::snake::builder::Builder as SnakeBuilder;

#[macro_use]
pub mod spawn;

#[derive(Debug, Clone)]
pub enum Type {
    Food(Food),
    SpawnSnake(Box<SnakeBuilder>),
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

#[derive(Clone)]
pub struct Apple {
    pub pos: HexPoint,
    pub apple_type: Type,
}
