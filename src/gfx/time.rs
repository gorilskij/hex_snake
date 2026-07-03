//! A macroquad-backed monotonic clock that works on wasm (where
//! `std::time::Instant::now()` panics). `Duration` from `std` is fine on wasm —
//! only the clock source needs replacing — so this only reimplements `Instant`.

use std::ops::Sub;
use std::time::Duration;

use macroquad::time::get_time;

/// Drop-in replacement for `std::time::Instant`, measured in seconds since the
/// macroquad context started.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Instant(f64);

impl Instant {
    pub fn now() -> Self {
        Instant(get_time())
    }

    pub fn elapsed(&self) -> Duration {
        Duration::from_secs_f64((get_time() - self.0).max(0.0))
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        Duration::from_secs_f64((self.0 - earlier.0).max(0.0))
    }

    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        if self.0 >= earlier.0 {
            Some(Duration::from_secs_f64(self.0 - earlier.0))
        } else {
            None
        }
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, rhs: Duration) -> Instant {
        Instant(self.0 - rhs.as_secs_f64())
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, rhs: Instant) -> Duration {
        self.duration_since(rhs)
    }
}

impl std::ops::Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Instant {
        Instant(self.0 + rhs.as_secs_f64())
    }
}
