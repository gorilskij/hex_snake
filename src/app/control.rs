use crate::{app::game_context::GameContext, basic::FrameStamp};
use std::{
    cmp::max,
    collections::VecDeque,
    time::{Duration, Instant},
};

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
            self.buffer.push_back(NFrameInstant(
                self.step - self.n + num_frames - 1,
                Instant::now(),
            ));
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

// combines fps with game state management
pub struct Control {
    game_fps: f64,
    game_frame_duration: Duration,
    start: Instant,
    last_update: Instant,

    // amount of time which game frames have not yet
    // been accounted for (will be included next time
    // this in done)
    remainder: f64, // secs

    // number of game frames that still need to be
    // performed to catch up with the current time
    // TODO: zero this when it gets too large to prevent lag
    missed_updates: Option<usize>,

    // counting graphics frames
    graphics_frame_num: usize,

    // empirical measurement of framerates unrelated to
    // control mechanism
    measured_game_fps: FpsCounter,
    measured_graphics_fps: FpsCounter,

    game_state: State,

    // used to store the frame fraction when the game is paused
    frozen_frame_fraction: Option<f32>,
}

impl Control {
    pub fn new(fps: f64) -> Self {
        let now = Instant::now();
        Self {
            game_fps: fps,
            game_frame_duration: Duration::from_nanos((1_000_000_000.0 / fps) as u64),
            start: now,
            last_update: now,
            remainder: 0.,

            missed_updates: None,

            graphics_frame_num: 0,

            measured_game_fps: FpsCounter::new(fps),
            measured_graphics_fps: FpsCounter::new(60.),

            game_state: State::Playing,
            frozen_frame_fraction: None,
        }
    }

    pub fn fps(&self) -> f64 {
        self.game_fps
    }

    // adjust self.last_update to make it match the expected
    // frame_fraction, this is done when resuming a paused game
    // and when adjust fps to ensure smoothness
    fn set_last_update_to_match_frame_fraction(&mut self, frac: f32) {
        let mut elapsed = (frac - self.remainder as f32) * self.game_frame_duration.as_secs_f32();
        // slight tolerance
        if (-0.01..0.).contains(&elapsed) {
            elapsed = 0.;
        } else {
            assert!(elapsed >= 0., "elapsed ({}s) < 0", elapsed);
        }

        self.last_update = Instant::now() - Duration::from_secs_f32(elapsed);
    }

    pub fn set_game_fps(&mut self, fps: f64) {
        if (self.game_fps - fps).abs() < f64::EPSILON {
            return;
        }

        // freeze frame fraction
        let frame_fraction = self.frame_fraction();

        self.game_fps = fps;
        self.game_frame_duration = Duration::from_nanos((1_000_000_000.0 / fps) as u64);
        self.measured_game_fps.set_expected_fps(fps);

        // revert to saved frame fraction
        self.set_last_update_to_match_frame_fraction(frame_fraction);
    }

    // repeatedly called in update() as while loop condition
    // WARN: this will perform as many updates as the framerate requires
    //  this can cause strong lag a high framerates
    // TODO: automatically lower game framerate to keep up graphics framerate
    pub fn can_update(&mut self) -> bool {
        if self.game_state != State::Playing {
            return false;
        }

        match &mut self.missed_updates {
            Some(0) => {
                self.missed_updates = None;
                false
            }
            Some(n) => {
                *n -= 1;
                true
            }
            None => {
                // calculate how many game frames should have occurred
                // since the last call to can_update
                let game_frames = self.last_update.elapsed().as_secs_f64()
                    / self.game_frame_duration.as_secs_f64()
                    + self.remainder;
                let missed_updates = game_frames as usize;

                if missed_updates > 0 {
                    self.remainder = game_frames % 1.;
                    self.last_update = Instant::now();

                    self.missed_updates = Some(missed_updates - 1);

                    self.measured_game_fps.register_frames(missed_updates);

                    true
                } else {
                    false
                }
            }
        }
    }

    // call in draw()
    pub fn graphics_frame(&mut self, gtx: &mut GameContext) {
        self.measured_graphics_fps.register_frames(1);
        self.graphics_frame_num += 1;
        gtx.frame_stamp = self.frame_stamp();
        gtx.elapsed_millis = self.start.elapsed().as_millis();
    }

    pub fn state(&self) -> State {
        self.game_state
    }

    pub fn play(&mut self) {
        self.game_state = State::Playing;
        self.measured_game_fps.reset();
        match self.frozen_frame_fraction.take() {
            None => (),
            Some(frac) => self.set_last_update_to_match_frame_fraction(frac),
        }
    }

    pub fn pause(&mut self) {
        self.game_state = State::Paused;
        self.frozen_frame_fraction = Some(self.frame_fraction());
        self.missed_updates = None;
    }

    pub fn game_over(&mut self) {
        self.game_state = State::GameOver;
        self.frozen_frame_fraction = Some(self.frame_fraction());
    }

    // fraction of the current game frame that has elapsed
    pub fn frame_fraction(&self) -> f32 {
        match self.frozen_frame_fraction {
            Some(frac) => frac,
            None => {
                let frac = self.last_update.elapsed().as_secs_f32()
                    / self.game_frame_duration.as_secs_f32()
                    + self.remainder as f32;
                if frac > 1. {
                    eprintln!("warning: frame fraction > 1 ({})", frac);
                    1.
                } else {
                    frac
                }
            }
        }
    }

    pub fn frame_stamp(&self) -> FrameStamp {
        (self.graphics_frame_num, self.frame_fraction())
    }

    pub fn measured_game_fps(&self) -> f64 {
        self.measured_game_fps.fps()
    }

    pub fn measured_graphics_fps(&self) -> f64 {
        self.measured_graphics_fps.fps()
    }
}
