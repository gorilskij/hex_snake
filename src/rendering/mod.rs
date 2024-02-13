pub use apple_mesh::apple_mesh;
pub use button_mesh::palette_changing_buttons_mesh;
pub use grid_mesh::{border_mesh, grid_dot_mesh, grid_mesh};
pub use player_path_mesh::player_path_mesh;
pub use snake_mesh::snake_mesh;

mod apple_mesh;
mod button_mesh;
mod clean_arc;
mod grid_mesh;
mod player_path_mesh;
pub mod segments;
pub mod shape;
mod snake_mesh;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Style {
    Hexagon,
    Smooth,
}
