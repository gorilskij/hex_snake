#[allow(dead_code)]
pub enum Effect {
    SmallHex {
        min_scale: f32,
        iterations: usize, // total number of iterations it will take
        passed: usize, // iterations passed so far
    }
}