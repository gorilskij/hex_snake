pub use other_snakes::OtherSnakes;
use rayon::prelude::ParallelIterator;

use crate::snake::{Segment, Snake};

mod all_snakes;
mod other_snakes;

// pub struct BoxedParallelIterator<I: ParallelIterator>(I);

pub trait ObjectSafeParallelIterator {
    type Item;
}

impl<I: ParallelIterator> ObjectSafeParallelIterator for I {
    type Item = I::Item;
}

pub trait Snakes {
    // fn nth(&self, n: usize) -> &Snake;
    // fn nth_mut(&mut self, n: usize) -> &mut Snake;

    fn iter(&self) -> Box<dyn Iterator<Item = &Snake> + '_>;
    // fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a>;

    fn par_iter(&self) -> Box<dyn ObjectSafeParallelIterator<Item = &Snake> + '_>;

    // fn par_iter_mut(&self) -> Box<dyn ParallelIterator<Item = &mut Snake>>;

    // fn iter_bodies(&self) -> Box<dyn Iterator<Item = &Body>> {
    //     Box::new(self.iter().map(|Snake { body, .. }| body))
    // }
    //
    // fn iter_bodies_mut(&self) -> impl Iterator<Item = &mut Body> {
    //     self.iter_mut().map(|Snake { body, .. }| body)
    // }
    //
    // fn par_iter_bodies(&self) -> impl ParallelIterator<Item = &Body> {
    //     self.par_iter().map(|Snake { body, .. }| body)
    // }
    //
    // fn par_iter_bodies_mut(&self) -> impl ParallelIterator<Item = &mut Body> {
    //     self.par_iter_mut().map(|Snake { body, .. }| body)
    // }

    fn iter_segments(&self) -> Box<dyn Iterator<Item = &Segment> + '_> {
        Box::new(self.iter().flat_map(|snake| snake.body.segments.iter()))
    }
    //
    // fn iter_segments_mut<'a>(&'a mut self) -> Self::IterSegmentsMut<'a> {
    //     self.iter_mut().flat_map(|snake| snake.body.segments.iter_mut())
    // }

    //
    // fn par_iter_segments(&self) -> impl ParallelIterator<Item = &Segment> {
    //     self.par_iter_bodies()
    //         .flat_map(|body| body.cells.par_iter())
    // }
}

impl Snakes for &dyn Snakes {
    fn iter(&self) -> Box<dyn Iterator<Item = &Snake> + '_> {
        (*self).iter()
    }

    fn par_iter(&self) -> Box<dyn ObjectSafeParallelIterator<Item = &Snake> + '_> {
        (*self).par_iter()
    }
}
