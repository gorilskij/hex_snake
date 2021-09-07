use crate::app::screen::prefs::Food;
use crate::app::snake;
use crate::basic::HexPoint;

#[derive(Debug)]
pub enum AppleType {
    Normal(Food),
    SpawnSnake(snake::Seed),
}

pub struct Apple {
    pub pos: HexPoint,
    pub typ: AppleType,
}