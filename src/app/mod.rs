pub use palette::Palette;

#[cfg(test)]
mod benchmark;
mod border_hints;
pub(crate) mod distance_grid;
pub(crate) mod fps_control;
pub mod game_context;
pub mod game_mode;
pub mod key;
pub mod message;
mod palette;
pub mod portal;
pub(crate) mod prefs;
pub(crate) mod screen;
mod snake_management;
pub mod stats;
