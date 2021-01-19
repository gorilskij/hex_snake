// an iterator that repeats each element n times
pub struct Iter<T, I> {
    times: usize,
    current: Option<(T, usize)>,
    iter: I,
}

impl<T: Clone, I: Iterator<Item = T>> Iterator for Iter<T, I> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let None | Some((_, 0)) = self.current {
            self.current = Some((self.iter.next()?, self.times));
        }

        let (val, n) = self.current.as_mut()?;
        *n -= 1;
        Some(val.clone())
    }
}

pub trait Times<T, I> {
    fn times(self, times: usize) -> Iter<T, I>;
}

impl<T: Clone, I: Iterator<Item = T>> Times<T, I> for I {
    fn times(self, times: usize) -> Iter<T, I> {
        Iter {
            times,
            current: None,
            iter: self,
        }
    }
}
