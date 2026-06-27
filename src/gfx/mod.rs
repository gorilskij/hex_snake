//! Thin compatibility layer that mimics the small subset of the `ggez` API the
//! game uses, backed by `macroquad` (so the game runs natively *and* on wasm).
//!
//! The module paths mirror ggez (`gfx::graphics`, `gfx::input::keyboard`,
//! `gfx::event`, `gfx::Context`, `gfx::GameError`) so porting a file is mostly a
//! matter of repointing its `use ggez::…` lines at `crate::gfx::…`.

use std::fmt::{self, Display, Formatter};

use macroquad::input::mouse_position;
use macroquad::window::{screen_height, screen_width};

use crate::basic::Point;

pub mod event;
pub mod graphics;
pub mod input;
pub mod time;

/// Opaque rendering context. In ggez this carried the GPU device, window, etc.
/// Under macroquad the global state lives in the framework, so this is mostly a
/// handle used to query the window size and mouse.
pub struct Context {
    pub gfx: GraphicsContext,
    pub mouse: MouseContext,
}

impl Context {
    pub fn new() -> Self {
        Self {
            gfx: GraphicsContext,
            mouse: MouseContext,
        }
    }
}

pub struct MouseContext;

impl MouseContext {
    pub fn position(&self) -> Point {
        let (x, y) = mouse_position();
        Point { x, y }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GraphicsContext;

impl GraphicsContext {
    /// Drawable (pixel) size of the window.
    pub fn drawable_size(&self) -> (f32, f32) {
        (screen_width(), screen_height())
    }

    pub fn window(&self) -> Window {
        Window
    }
}

pub struct Window;

impl Window {
    pub fn inner_size(&self) -> WindowSize {
        WindowSize {
            width: screen_width(),
            height: screen_height(),
        }
    }
}

pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

impl From<WindowSize> for Point {
    fn from(size: WindowSize) -> Self {
        Point { x: size.width, y: size.height }
    }
}

/// Stand-in for `ggez::GameError`. The compat layer is infallible, but the
/// graphics methods keep returning `Result<_, GameError>` so the existing
/// `?`-and-`map_err(Error::from)` call sites compile unchanged.
#[derive(Debug)]
pub struct GameError(pub String);

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "GameError: {}", self.0)
    }
}

impl std::error::Error for GameError {}

pub type GameResult<T = ()> = Result<T, GameError>;
