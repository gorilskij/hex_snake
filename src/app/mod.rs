pub use palette::Palette;

#[cfg(test)]
mod benchmark;
pub(crate) mod distance_grid;
pub(crate) mod fps_control;
pub mod game_context;
pub mod game_mode;
pub mod key;
mod light_hints;
pub mod message;
mod palette;
pub mod portal;
pub(crate) mod prefs;
pub(crate) mod screen;
mod snake_management;
pub mod stats;
