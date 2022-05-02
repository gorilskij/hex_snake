use crate::app::snake;
use ggez::GameError;
use std::{
    error::Error,
    fmt,
    fmt::{Debug, Display, Formatter},
};

#[derive(Debug)]
pub enum AppErrorType {
    GameError(GameError),
    SnakeBuilderError(snake::BuilderError),
}

/// The second member contains a trace in reverse order
#[must_use]
pub struct AppError(AppErrorType, Vec<String>);

impl From<GameError> for AppError {
    fn from(e: GameError) -> Self {
        Self(AppErrorType::GameError(e), vec![])
    }
}

impl From<snake::BuilderError> for AppError {
    fn from(e: snake::BuilderError) -> Self {
        Self(AppErrorType::SnakeBuilderError(e), vec![])
    }
}

impl AppError {
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
        Debug::fmt(self, f)
    }
}

impl Error for AppError {}

pub type AppResult<T = ()> = Result<T, AppError>;

pub trait AppErrorConversion {
    fn with_trace_step<S: ToString>(self, s: S) -> Self;
}

impl<T> AppErrorConversion for AppResult<T> {
    fn with_trace_step<S: ToString>(self, s: S) -> Self {
        self.map_err(|e| e.with_trace_step(s.to_string()))
    }
}
