//! Thin compatibility layer that mimics the small subset of the `ggez` API the
//! game uses, backed by `macroquad` (so the game runs natively *and* on wasm).
//!
//! The module paths mirror ggez (`gfx::graphics`, `gfx::input::keyboard`,
//! `gfx::event`, `gfx::Context`, `gfx::GameError`) so porting a file is mostly a
//! matter of repointing its `use ggez::…` lines at `crate::gfx::…`.

use std::fmt::{self, Display, Formatter};

pub mod event;
pub mod graphics;
pub mod input;
pub mod material;
pub mod time;

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
