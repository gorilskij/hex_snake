use crate::app::snake::Snake;
use crate::app::hex::Hex;
use crate::app::palette::SnakePalette;

pub struct SimSnake {
    body: Vec<Hex>,
    palette: SnakePalette,
}

impl Snake for SimSnake {
    fn body(&self) -> &Vec<Hex> {
        &self.body
    }

    fn palette(&self) -> &SnakePalette {
        &self.palette
    }

    fn advance(&mut self) {
        unimplemented!()
    }
}