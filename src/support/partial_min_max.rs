use itertools::MinMaxResult;
use itertools::MinMaxResult::MinMax;
use std::cmp::Ordering;

// the partial functions return None in case of a failed comparison

// TODO: return on first None
pub trait PartialMinMax
where
    Self: Iterator + Sized,
{
    fn partial_minmax(mut self) -> MinMaxResult<Self::Item>
    where
        Self::Item: PartialOrd,
    {
        use MinMaxResult::*;

        let first = match self.next() {
            None => NoElements,
            Some(x) => OneElement(x),
        };
        self.fold(first, |x, y| match x {
            OneElement(a) => {
                if y < a {
                    MinMax(y, a)
                } else if y > a {
                    MinMax(a, y)
                } else {
                    OneElement(a)
                }
            }
            MinMax(a, b) => {
                if y < a {
                    MinMax(y, b)
                } else if y > b {
                    MinMax(a, y)
                } else {
                    MinMax(a, b)
                }
            }
            NoElements => unreachable!(),
        })
    }

    fn partial_minmax_copy(self) -> Option<(Self::Item, Self::Item)>
    where
        Self::Item: PartialOrd + Copy,
    {
        match self.partial_minmax() {
            MinMaxResult::NoElements => None,
            MinMaxResult::OneElement(a) => Some((a, a)),
            MinMax(a, b) => Some((a, b)),
        }
    }

    fn partial_min_by_key<B, F>(self, mut f: F) -> Option<Self::Item>
    where
        F: FnMut(&Self::Item) -> B,
        B: PartialOrd,
    {
        let mut mapped = self.map(|x| (f(&x), x));
        let first = mapped.next();
        mapped
            .fold(first, |ox, (fy, y)| {
                let (fx, x) = ox?;
                match fx.partial_cmp(&fy)? {
                    Ordering::Less | Ordering::Equal => Some((fx, x)),
                    Ordering::Greater => Some((fy, y)),
                }
            })
            .map(|(_, x)| x)
    }
}

impl<I: Iterator> PartialMinMax for I {}

#[allow(dead_code)]
pub fn partial_min<T: PartialOrd>(a: T, b: T) -> Option<T> {
    match a.partial_cmp(&b)? {
        Ordering::Less | Ordering::Equal => Some(a),
        Ordering::Greater => Some(b),
    }
}

#[allow(dead_code)]
pub fn partial_max<T: PartialOrd>(a: T, b: T) -> Option<T> {
    match a.partial_cmp(&b)? {
        Ordering::Greater => Some(a),
        Ordering::Less | Ordering::Equal => Some(b),
    }
}
