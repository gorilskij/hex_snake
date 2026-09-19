/// Which rules a game is played by.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GameMode {
    /// Apples are eaten and digested (eaten segments you can pass through);
    /// snakes never shrink on their own.
    Classic,
    /// Apples grow snakes right away, apple-eating snakes shrink steadily, and
    /// bad apples shrink them further. The player starving down to
    /// [`hunger::MIN_LENGTH`] is game over.
    Hunger,
}

impl GameMode {
    /// How fast apple-eating snakes shrink on their own, in cells/s.
    pub fn starvation(self) -> f32 {
        match self {
            GameMode::Classic => 0.,
            GameMode::Hunger => hunger::STARVATION,
        }
    }
}

/// Tuning for [`GameMode::Hunger`].
pub mod hunger {
    use std::ops::Range;
    use std::time::Duration;

    /// How fast apple-eating snakes (player and competitors) shrink, in cells/s.
    pub const STARVATION: f32 = 0.3;
    /// A snake this short has starved.
    pub const MIN_LENGTH: f32 = 1.;

    /// Growth from an apple, in cells.
    pub const GROW: f32 = 1.;
    /// Seconds over which an apple's growth is applied. It eases out, so at
    /// first growth outpaces the snake and the tail moves backwards.
    pub const GROW_DURATION: f32 = 1.;

    /// A bad apple shrinks by a random amount in this range, in cells.
    pub const BAD_APPLE_SHRINK: Range<f32> = 3.0..5.0;
    /// Seconds over which a bad apple's shrinking is applied.
    pub const SHRINK_DURATION: f32 = 1.;
    /// Average seconds between bad apples (they appear at random times).
    pub const BAD_APPLE_INTERVAL: f32 = 10.;
    pub const MAX_BAD_APPLES: usize = 3;
    /// Bad apples disappear after this much game time.
    pub const BAD_APPLE_LIFETIME: Duration = Duration::from_secs(5);
}
