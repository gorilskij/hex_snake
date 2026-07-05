//! Small graphics layer over `macroquad` (so the game runs natively *and* on
//! wasm): mesh building + tessellation (`graphics`), the snake shader material
//! (`material`), the `EventHandler` trait (`event`), and a wasm-safe monotonic
//! clock (`time`).

pub mod event;
pub mod graphics;
pub mod material;
pub mod time;
