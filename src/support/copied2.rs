pub trait Copied2<I> {
    fn copied2(self) -> Iter<I>;
}

pub struct Iter<I>(I);

impl<'a, 'b, T: 'a + Copy, U: 'b + Copy, I: Iterator<Item = (&'a T, &'b U)>> Iterator for Iter<I> {
    type Item = (T, U);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(x, y)| (*x, *y))
    }
}

impl<'a, 'b, T: 'a + Copy, U: 'b + Copy, I: Iterator<Item = (&'a T, &'b U)>> Copied2<Self> for I {
    fn copied2(self) -> Iter<Self> {
        Iter(self)
    }
}
