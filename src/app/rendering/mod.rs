pub use apple_mesh::apple_mesh;
pub use grid_mesh::{border_mesh, grid_mesh};
pub use snake_mesh::snake_mesh;

mod apple_mesh;
mod grid_mesh;
pub mod segments;
mod snake_mesh;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Style {
    Hexagon,
    Smooth,
}
