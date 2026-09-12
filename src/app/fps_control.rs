use std::cmp::max;
use std::collections::VecDeque;
use std::time::Duration;

use crate::support::time::Instant;

/// Stores an instant along with the number of frames it represents
struct NFrameInstant(usize, Instant);

/// Objective measurement of framerate based on periodic calls
/// to [`FpsCounter::register_frame`], completely detached from any
/// framerate-regulation mechanism
struct FpsCounter {
    /// An `Instant` is stored every `step` frames. This isn't
    /// done every frame because calling `Instant::now()` produces
    /// a syscall which is slow.
    step: usize,
    /// Counts down from `step` to 0 to tell when the next
    /// `Instant` should be stored
    n: usize,
    /// A queue of `NFrameInstant`s
    buffer: VecDeque<NFrameInstant>,
}

impl FpsCounter {
    /// Number of `Instant`s to store in `buffer`
    const LEN: usize = 10;

    fn new(expected_fps: f64) -> Self {
        let mut counter = Self {
            step: 0,
            n: 0,
            buffer: VecDeque::with_capacity(Self::LEN),
        };
        counter.set_expected_fps(expected_fps);
        counter
    }

    /// `FpsCounter` knows about the expected framerate to adjust
    /// how often it stores an `Instant`, for low framerates this
    /// is done every few frames but for higher framerates, it's
    /// much rarer to avoid thousands of syscalls per second
    fn set_expected_fps(&mut self, expected_fps: f64) {
        // store an instant ~every N seconds, but at most every frame
        const N: f64 = 1.;
        self.step = max(1, (expected_fps * N) as usize);
        self.reset();
    }

    fn register_frames(&mut self, num_frames: usize) {
        if self.n < num_frames {
            if self.buffer.len() >= Self::LEN {
                self.buffer.pop_front();
            }
            self.buffer
                .push_back(NFrameInstant(self.step - self.n + num_frames - 1, Instant::now()));
            self.n = self.step - 1;
        } else {
            self.n -= num_frames;
        }
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.n = 0;
    }

    /// The framerate is calculated as the inverse of the
    /// average frame duration
    fn fps(&self) -> f64 {
        if self.buffer.len() >= 2 {
            let first_frame = self.buffer[0].1;
            let last_frame = self.buffer[self.buffer.len() - 1].1;
            let total_buffer_duration = (last_frame - first_frame).as_secs_f64();
            // let num_frames = ((self.buffer.len() - 1) * self.step) as f64;
            let num_frames = self.buffer.iter().skip(1).map(|nfi| nfi.0).sum::<usize>() as f64;
            num_frames / total_buffer_duration
        } else {
            0.
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum State {
    Playing,
    Paused,
    GameOver,
}

/// Smoothing factor for the frame-duration average: each frame contributes this
/// much of itself. Lower is smoother but slower to follow a real rate change.
const PACE_SMOOTHING: f64 = 0.1;

/// Longest hitch (relative to the running average) that still advances the
/// world. A frame longer than this is a real stall — window drag, a breakpoint —
/// and the world advances by this much instead of teleporting.
const MAX_PACE_RATIO: f64 = 4.;

// combines fps with game state management
pub struct FpsControl {
    game_state: State,

    start: Instant,

    last_update: Instant,

    /// Running average of recent frame durations — the duration the world
    /// actually advances by. See [`FpsControl::pace`].
    paced: Option<Duration>,

    graphics_frame_num: usize,
    elapsed_total: Duration,
    measured_graphics_fps: FpsCounter,
}

impl FpsControl {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            game_state: State::Playing,

            start: now,

            last_update: now,

            paced: None,

            graphics_frame_num: 0,
            elapsed_total: Duration::ZERO,
            measured_graphics_fps: FpsCounter::new(60.),
        }
    }

    pub fn update(&mut self) -> Option<Duration> {
        (self.game_state == State::Playing).then(|| {
            let new_update = Instant::now();
            let elapsed = new_update - self.last_update;
            self.last_update = new_update;
            self.pace(elapsed)
        })
    }

    /// Smooth a raw frame duration into the one the world advances by.
    ///
    /// Frames are presented on the display's fixed cadence, but the wall-clock
    /// gap we measure between `update` calls jitters around it by a few percent
    /// — scheduling noise, not real elapsed time. Advancing the world by that
    /// noisy measurement makes everything move at a slightly wrong speed every
    /// frame: invisible on the snake's uniform body, but clearly visible on the
    /// head and tail tips, which are the only features the eye can track.
    ///
    /// A running average removes the noise while still following a genuine rate
    /// change (a different monitor, vsync off). It is unbiased, so simulated
    /// time tracks wall time; a dropped frame is absorbed over the next few
    /// frames rather than lurching.
    ///
    /// Deliberately *not* snapping to whole refresh periods: the period has to
    /// be estimated from the same jittery samples, so the snap target wobbles,
    /// and frames near 1.5x round up into an advance that never happened. Both
    /// measured worse than doing nothing.
    fn pace(&mut self, raw: Duration) -> Duration {
        let Some(avg) = self.paced else {
            // first frame: nothing to average against yet
            self.paced = Some(raw);
            return raw;
        };

        // a real stall must not poison the average (or teleport the snake)
        let capped = raw.min(avg.mul_f64(MAX_PACE_RATIO));

        let smoothed = avg.mul_f64(1. - PACE_SMOOTHING) + capped.mul_f64(PACE_SMOOTHING);
        self.paced = Some(smoothed);
        smoothed
    }

    // call in draw()
    pub fn graphics_frame(&mut self) {
        self.measured_graphics_fps.register_frames(1);
        self.elapsed_total = self.start.elapsed();
        self.graphics_frame_num += 1;
    }

    pub fn elapsed_total(&self) -> Duration {
        self.elapsed_total
    }

    pub fn state(&self) -> State {
        self.game_state
    }

    pub fn play(&mut self) {
        self.game_state = State::Playing;
        self.last_update = Instant::now();
    }

    pub fn pause(&mut self) {
        self.game_state = State::Paused;
    }

    pub fn game_over(&mut self) {
        self.game_state = State::GameOver;
    }

    pub fn measured_graphics_fps(&self) -> f64 {
        self.measured_graphics_fps.fps()
    }
}
