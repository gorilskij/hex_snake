use crate::app::snake::{Body, Segment, Snake};
use rayon::prelude::*;

#[derive(Copy, Clone)]
pub struct OtherSnakes<'a>(&'a [Snake], &'a [Snake]);

#[allow(dead_code)]
impl<'a> OtherSnakes<'a> {
    pub fn empty() -> Self {
        Self(&[], &[])
    }

    pub fn new(a: &'a [Snake], b: &'a [Snake]) -> Self {
        Self(a, b)
    }

    pub fn iter_snakes(&self) -> impl Iterator<Item = &Snake> {
        self.0.iter().chain(self.1.iter())
    }

    pub fn par_iter_snakes(&self) -> impl ParallelIterator<Item = &Snake> {
        self.0.par_iter().chain(self.1.par_iter())
    }

    pub fn iter_bodies(&self) -> impl Iterator<Item = &Body> {
        self.iter_snakes().map(|Snake { body, .. }| body)
    }

    pub fn par_iter_bodies(&self) -> impl ParallelIterator<Item = &Body> {
        self.par_iter_snakes().map(|Snake { body, .. }| body)
    }

    pub fn iter_segments(&self) -> impl Iterator<Item = &Segment> {
        self.iter_bodies().flat_map(|body| body.cells.iter())
    }

    pub fn par_iter_segments(&self) -> impl ParallelIterator<Item = &Segment> {
        self.par_iter_bodies()
            .flat_map(|body| body.cells.par_iter())
    }
}

/// Extract one snake at `idx` and return all other
/// snakes in a special struct to avoid building
/// unnecessary vecs all the time (OtherSnakes is always
/// immutable)
pub fn split_snakes_mut(snakes: &mut [Snake], idx: usize) -> (&mut Snake, OtherSnakes) {
    let (other_snakes1, rest) = snakes.split_at_mut(idx);
    let (snake, other_snakes2) = rest.split_first_mut().unwrap();
    (snake, OtherSnakes::new(other_snakes1, other_snakes2))
}
