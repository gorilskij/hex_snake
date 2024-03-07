pub trait FilterScan
where
    Self: Iterator,
{
    fn filter_scan<St, B, F>(self, initial_state: St, f: F) -> Iter<Self, St, F>
    where
        Self: Sized,
        F: FnMut(&mut St, Self::Item) -> Option<B>;
}

impl<T, I: Iterator<Item = T>> FilterScan for I {
    fn filter_scan<St, B, F>(self, initial_state: St, f: F) -> Iter<Self, St, F>
    where
        Self: Sized,
        F: FnMut(&mut St, Self::Item) -> Option<B>,
    {
        Iter { iter: self, state: initial_state, f }
    }
}

pub struct Iter<I, St, F> {
    iter: I,
    state: St,
    f: F,
}

impl<B, I, St, F> Iterator for Iter<I, St, F>
where
    I: Iterator,
    F: FnMut(&mut St, <I as Iterator>::Item) -> Option<B>,
{
    type Item = B;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = (self.f)(&mut self.state, self.iter.next()?);
            if item.is_some() {
                return item;
            }
        }
    }
}
