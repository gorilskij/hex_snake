pub trait Flip {
    fn flip(&mut self) -> Self;
}

impl Flip for bool {
    fn flip(&mut self) -> Self {
        *self ^= true;
        *self
    }
}
