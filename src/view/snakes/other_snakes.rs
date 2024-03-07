use rayon::prelude::*;

use crate::snake::Snake;
use crate::view::snakes::{ObjectSafeParallelIterator, Snakes};

#[derive(Copy, Clone)]
pub struct OtherSnakes<'a>(&'a [Snake], &'a [Snake]);

#[allow(dead_code)]
impl<'a> OtherSnakes<'a> {
    pub fn empty() -> Self {
        Self(&[], &[])
    }

    // pub fn new(a: &'a [Snake], b: &'a [Snake]) -> Self {
    //     Self(a, b)
    // }

    pub fn split_snakes(snakes: &mut [Snake], idx: usize) -> (&mut Snake, OtherSnakes) {
        let (other_snakes1, rest) = snakes.split_at_mut(idx);
        let (snake, other_snakes2) = rest.split_first_mut().unwrap();
        (snake, OtherSnakes(other_snakes1, other_snakes2))
    }

    // pub fn par_iter_snakes(&self) -> impl ParallelIterator<Item = &Snake> {
    //     self.0.par_iter().chain(self.1.par_iter())
    // }

    // pub fn iter_bodies(&self) -> impl Iterator<Item = &Body> {
    //     self.iter_snakes().map(|Snake { body, .. }| body)
    // }
    //
    // pub fn par_iter_bodies(&self) -> impl ParallelIterator<Item = &Body> {
    //     self.par_iter_snakes().map(|Snake { body, .. }| body)
    // }
    //
    // pub fn iter_segments(&self) -> impl Iterator<Item = &Segment> {
    //     self.iter_bodies().flat_map(|body| body.cells.iter())
    // }
    //
    // pub fn par_iter_segments(&self) -> impl ParallelIterator<Item = &Segment> {
    //     self.par_iter_bodies()
    //         .flat_map(|body| body.cells.par_iter())
    // }
}

impl Snakes for OtherSnakes<'_> {
    fn iter(&self) -> Box<dyn Iterator<Item = &Snake> + '_> {
        Box::new(self.0.iter().chain(self.1.iter()))
    }

    fn par_iter(&self) -> Box<dyn ObjectSafeParallelIterator<Item = &Snake> + '_> {
        Box::new(self.0.par_iter().chain(self.1.par_iter()))
    }

    // fn nth(&self, n: usize) -> &Snake {
    //     if n < self.0.len() {
    //         &self.0[n]
    //     } else {
    //         &self.1[n - self.0.len()]
    //     }
    // }
    //
    // fn nth_mut(&mut self, n: usize) -> &mut Snake {
    //     if n < self.0.len() {
    //         &mut self.0[n]
    //     } else {
    //         &mut self.1[n - self.0.len()]
    //     }
    // }

    // fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a> {
    //     self.0.iter_mut().chain(self.1.iter_mut())
    // }
}
