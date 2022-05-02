/// Ref or Owned, pretty specific utility type
/// for when a reference is needed but it might
/// also be necessary to own a value
pub enum ROw<'a, T: 'a> {
    Ref(&'a T),
    Owned(T),
}

impl<'a, T: 'a> ROw<'a, T> {
    pub fn get(&self) -> &T {
        match self {
            Self::Ref(x) => *x,
            Self::Owned(ref x) => x,
        }
    }
}
