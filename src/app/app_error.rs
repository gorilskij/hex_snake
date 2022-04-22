use ggez::{GameError, GameResult};
use std::{
    error::Error,
    fmt,
    fmt::{Debug, Display, Formatter},
};

/// The second member contains a trace in reverse order
pub struct AppError(GameError, Vec<String>);

impl From<GameError> for AppError {
    fn from(e: GameError) -> Self {
        Self::from_game_error(e)
    }
}

impl AppError {
    pub fn from_game_error(e: GameError) -> Self {
        Self(e, vec![])
    }

    pub fn with_trace_step<S: ToString>(mut self, s: S) -> Self {
        self.1.push(s.to_string());
        self
    }
}

impl Debug for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error:\n{:?}\nTrace:", self.0)?;
        for t in (self.1).iter().rev() {
            writeln!(f, " in {}", t)?;
        }
        Ok(())
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error:\n{}\nTrace:", self.0)?;
        for t in (self.1).iter().rev() {
            writeln!(f, " in {}", t)?;
        }
        Ok(())
    }
}

impl Error for AppError {}

pub type AppResult<T = ()> = Result<T, AppError>;

pub trait AppErrorConversion {
    fn with_trace_step<S: ToString>(self, s: S) -> Self;
}

impl<T> AppErrorConversion for AppResult<T> {
    fn with_trace_step<S: ToString>(mut self, s: S) -> Self {
        self.map_err(|e| e.with_trace_step(s.to_string()))
    }
}

pub trait GameResultExtension<T> {
    fn into_with_trace<S: ToString>(self, s: S) -> AppResult<T>;
}

impl<T> GameResultExtension<T> for GameResult<T> {
    fn into_with_trace<S: ToString>(self, s: S) -> AppResult<T> {
        self.map_err(AppError::from_game_error).with_trace_step(s)
    }
}
