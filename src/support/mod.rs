pub mod map_with_default;

use std::collections::VecDeque;

pub trait Limits {
    type Value;

    fn first(&self) -> Option<&Self::Value>;

    fn first_mut(&mut self) -> Option<&mut Self::Value>;

    fn last(&self) -> Option<&Self::Value>;

    fn last_mut(&mut self) -> Option<&mut Self::Value>;
}

impl<T> Limits for VecDeque<T> {
    type Value = T;

    fn first(&self) -> Option<&Self::Value> {
        self.get(0)
    }

    fn first_mut(&mut self) -> Option<&mut Self::Value> {
        self.get_mut(0)
    }

    fn last(&self) -> Option<&Self::Value> {
        self.get(self.len() - 1)
    }

    fn last_mut(&mut self) -> Option<&mut Self::Value> {
        self.get_mut(self.len() - 1)
    }
}
