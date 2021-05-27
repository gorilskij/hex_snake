use ggez::graphics::Color;
use num_traits::Float;

pub mod oklab;

pub fn exclusive_linspace<F: Float>(a: F, b: F, n: usize) -> impl Iterator<Item=F> {
    itertools_num::linspace(a, b, n).take(n - 1)
}
