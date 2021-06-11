use std::cmp::Ordering;

// the partial functions return None in case of a failed comparison
pub trait PartialMinMax
where
    Self: Iterator + Sized,
{
    // this could be more efficient if it stopped on the first None value but it's meant to be used where all items are comparable
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

pub fn partial_min<T: PartialOrd>(a: T, b: T) -> Option<T> {
    match a.partial_cmp(&b)? {
        Ordering::Less | Ordering::Equal => Some(a),
        Ordering::Greater => Some(b),
    }
}

pub fn partial_max<T: PartialOrd>(a: T, b: T) -> Option<T> {
    match a.partial_cmp(&b)? {
        Ordering::Greater => Some(a),
        Ordering::Less | Ordering::Equal => Some(b),
    }
}
