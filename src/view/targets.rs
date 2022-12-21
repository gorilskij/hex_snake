use crate::apple::Apple;
use crate::basic::HexPoint;

pub trait Targets {
    fn nth(&self, n: usize) -> HexPoint;

    fn iter(&self) -> Box<dyn Iterator<Item = HexPoint> + '_>;
}

impl Targets for &[Apple] {
    fn nth(&self, n: usize) -> HexPoint {
        self[n].pos
    }

    fn iter(&self) -> Box<dyn Iterator<Item = HexPoint> + '_> {
        Box::new((*self).iter().map(|apple: &Apple| apple.pos))
    }
}
