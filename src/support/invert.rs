pub trait Invert<T, E> {
    fn invert(self) -> Result<Option<T>, E>;
}

impl<T, E> Invert<T, E> for Option<Result<T, E>> {
    fn invert(self) -> Result<Option<T>, E> {
        match self {
            None => Ok(None),
            Some(result) => result.map(Some),
        }
    }
}
