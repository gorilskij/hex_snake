use crate::{
    app::{snake, utils::Food},
    basic::HexPoint,
};

#[macro_use]
pub mod spawn;

#[derive(Debug)]
pub enum Type {
    Food(Food),
    SpawnSnake(Box<snake::Builder>),
    SpawnRain,
}

pub struct Apple {
    pub pos: HexPoint,
    pub apple_type: Type,
}
