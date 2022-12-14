use crate::snake;
use ggez::GameError;
use std::fmt::{Debug, Display, Formatter};
use std::{fmt, result};

#[derive(Debug)]
pub enum ErrorType {
    GameError(GameError),
    SnakeBuilderError(snake::BuilderError),
}

/// The second member contains a trace in reverse order
#[must_use]
pub struct Error(ErrorType, Vec<String>);

impl From<GameError> for Error {
    fn from(e: GameError) -> Self {
        Self(ErrorType::GameError(e), vec![])
    }
}

impl From<snake::BuilderError> for Error {
    fn from(e: snake::BuilderError) -> Self {
        Self(ErrorType::SnakeBuilderError(e), vec![])
    }
}

impl Error {
    pub fn with_trace_step<S: ToString>(mut self, s: S) -> Self {
        self.1.push(s.to_string());
        self
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error:\n{:?}\nTrace:", self.0)?;
        for t in (self.1).iter().rev() {
            writeln!(f, " in {}", t)?;
        }
        Ok(())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for Error {}

pub type Result<T = ()> = result::Result<T, Error>;

pub trait ErrorConversion {
    fn with_trace_step<S: ToString>(self, s: S) -> Self;
}

impl<T> ErrorConversion for Result<T> {
    fn with_trace_step<S: ToString>(self, s: S) -> Self {
        self.map_err(|e| e.with_trace_step(s.to_string()))
    }
}
