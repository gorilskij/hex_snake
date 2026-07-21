use crate::basic::HexPoint;
use crate::snake::builder::Builder as SnakeBuilder;

#[macro_use]
pub mod spawn;

#[derive(Debug, Clone)]
pub enum Type {
    Eat(f32),
    Shrink(f32),
    SpawnSnake(Box<SnakeBuilder>),
    SpawnRain,
}

impl Type {
    pub fn is_animated(&self) -> bool {
        // TODO: a) check if this is still relevant, b) make this dependent on palette
        true
        // match self {
        //     Type::Grow(_) => false,
        //     Type::SpawnSnake(_) => true,
        //     Type::SpawnRain => true,
        // }
    }
}

#[derive(Clone)]
pub struct Apple {
    pub pos: HexPoint,
    pub apple_type: Type,
}
