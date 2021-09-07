use spawning::SpawnPolicy;

use crate::app::snake;
use crate::basic::HexPoint;
use crate::app::utils::Food;

#[macro_use]
pub mod spawning;

#[derive(Debug)]
pub enum Type {
    Normal(Food),
    SpawnSnake(snake::Seed),
}

pub struct Apple {
    pub pos: HexPoint,
    pub apple_type: Type,
}
