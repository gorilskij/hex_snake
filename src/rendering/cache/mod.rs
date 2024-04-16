pub use snakes::Cache as SnakeGraphicsCache;

mod snakes;

#[derive(Default)]
pub struct GraphicsCache {
    pub snakes: snakes::Cache,
}
