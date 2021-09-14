mod apple_mesh;
mod grid_mesh;
mod snake_mesh;

pub use apple_mesh::apple_mesh;
pub use grid_mesh::{grid_mesh, border_mesh};
pub use snake_mesh::{snake_mesh};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Style {
    Hexagon,
    Smooth,
}
