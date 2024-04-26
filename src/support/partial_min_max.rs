use std::cmp::Ordering::*;

use itertools::MinMaxResult::{self, MinMax, *};

// the partial functions return None in case of a failed comparison

pub trait PartialMinMax
where
    Self: Iterator + Sized,
{
    // will return NoElements if a NaN is encountered
    fn partial_minmax(mut self) -> MinMaxResult<Self::Item>
    where
        Self::Item: PartialOrd,
    {
        let first = match self.next() {
            None => return NoElements,
            Some(x) => OneElement(x),
        };
        self.try_fold(first, |x, y| match x {
            OneElement(a) => match y.partial_cmp(&a)? {
                Less => Some(MinMax(y, a)),
                Equal => Some(OneElement(a)),
                Greater => Some(MinMax(a, y)),
            },
            MinMax(a, b) => Some(if y < a {
                MinMax(y, b)
            } else if y > b {
                MinMax(a, y)
            } else {
                MinMax(a, b)
            }),
            NoElements => unreachable!(),
        })
        .unwrap_or(NoElements)
    }

    fn partial_minmax_copy(self) -> Option<(Self::Item, Self::Item)>
    where
        Self::Item: PartialOrd + Copy,
    {
        match self.partial_minmax() {
            NoElements => None,
            OneElement(a) => Some((a, a)),
            MinMax(a, b) => Some((a, b)),
        }
    }

    fn partial_min_by_key<B, F>(mut self, mut f: F) -> Option<Self::Item>
    where
        F: FnMut(&Self::Item) -> B,
        B: PartialOrd,
    {
        let first = self.next()?;
        let f_first = f(&first);
        self.try_fold((first, f_first), |(x, fx), y| {
            let fy = f(&y);
            match fx.partial_cmp(&fy)? {
                Greater => Some((y, fy)),
                _ => Some((x, fx)),
            }
        })
        .map(|(x, _)| x)
    }
}

impl<I: Iterator> PartialMinMax for I {}

#[allow(dead_code)]
pub fn partial_min<T: PartialOrd>(a: T, b: T) -> Option<T> {
    match a.partial_cmp(&b)? {
        Greater => Some(b),
        _ => Some(a),
    }
}

#[allow(dead_code)]
pub fn partial_max<T: PartialOrd>(a: T, b: T) -> Option<T> {
    match a.partial_cmp(&b)? {
        Less => Some(b),
        _ => Some(a),
    }
}
