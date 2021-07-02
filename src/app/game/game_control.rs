use std::{
    cmp::max,
    collections::VecDeque,
    time::{Duration, Instant},
};
use crate::app::game::FrameStamp;

struct FPSCounter {
    step: usize, // record once every 'step' frames
    n: usize,    // counts down to recording the next frame
    buffer: VecDeque<Instant>,
}

impl FPSCounter {
    const LEN: usize = 10; // number of frame batches to store

    fn new(fps: f64) -> Self {
        let mut counter = Self {
            step: 0,
            n: 0,
            buffer: VecDeque::with_capacity(Self::LEN),
        };
        counter.set_expected_fps(fps);
        counter
    }

    // Counting parameters depend on the expected fps
    // (very high fps won't be updated as often)
    fn set_expected_fps(&mut self, fps: f64) {
        self.step = max(1, (3. * fps / Self::LEN as f64) as usize);
        // debug
        // println!(
        //     "FPSCounter: fps = {}, step = {}, len = {}, frames = {}",
        //     fps,
        //     self.step,
        //     Self::LEN,
        //     self.step * Self::LEN as Frames
        // );
        self.reset();
    }

    fn register_frame(&mut self) {
        if self.n == 0 {
            if self.buffer.len() >= Self::LEN {
                self.buffer.pop_front();
            }
            self.buffer.push_back(Instant::now());
            self.n = self.step - 1;
        } else {
            self.n -= 1;
        }
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.n = 0;
    }

    fn fps(&self) -> f64 {
        if self.buffer.len() >= 2 {
            ((self.buffer.len() - 1) as f64 * self.step as f64)
                / (self.buffer[self.buffer.len() - 1] - self.buffer[0]).as_secs_f64()
        } else {
            0.
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
}

// combines fps with game state management
pub struct GameControl {
    game_fps: f64,
    game_frame_duration: Duration,
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
    measured_game_fps: FPSCounter,
    measured_graphics_fps: FPSCounter,

    game_state: GameState,

    // used to store the frame fraction when the game is paused
    frozen_frame_fraction: Option<f32>,
}

impl GameControl {
    pub fn new(fps: f64) -> Self {
        Self {
            game_fps: fps,
            game_frame_duration: Duration::from_nanos((1_000_000_000.0 / fps) as u64),
            last_update: Instant::now(),
            remainder: 0.,

            missed_updates: None,

            graphics_frame_num: 0,

            measured_game_fps: FPSCounter::new(fps),
            measured_graphics_fps: FPSCounter::new(60.),

            game_state: GameState::Playing,
            frozen_frame_fraction: None,
        }
    }

    pub fn fps(&self) -> f64 {
        self.game_fps
    }

    // adjust self.last_update to make it match the expected
    // frame_fraction, this is done when resuming a paused game
    // and when adjust fps to ensure smoothness
    fn set_last_update_to_match_frame_frac(&mut self, frac: f32) {
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

        let frac = self.frame_fraction();

        self.game_fps = fps;
        self.game_frame_duration = Duration::from_nanos((1_000_000_000.0 / fps) as u64);
        self.measured_game_fps.set_expected_fps(fps);

        // keep frame_frac constant
        self.set_last_update_to_match_frame_frac(frac);
    }

    // repeatedly called in update() as while loop condition
    // WARN: this will perform as many updates as the framerate requires
    //  this can cause strong lag a high framerates
    // TODO: automatically lower game framerate to keep up graphics framerate
    pub fn can_update(&mut self) -> bool {
        if self.game_state != GameState::Playing {
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

                    // TODO: O(1)ize
                    for _ in 0..missed_updates {
                        self.measured_game_fps.register_frame();
                    }

                    true
                } else {
                    false
                }
            }
        }
    }

    // call in draw()
    pub fn graphics_frame(&mut self) {
        self.measured_graphics_fps.register_frame();
        self.graphics_frame_num += 1;
    }

    pub fn state(&self) -> GameState {
        self.game_state
    }

    pub fn play(&mut self) {
        self.game_state = GameState::Playing;
        self.measured_game_fps.reset();
        match self.frozen_frame_fraction.take() {
            None => (),
            Some(frac) => self.set_last_update_to_match_frame_frac(frac),
        }
    }

    pub fn pause(&mut self) {
        self.game_state = GameState::Paused;
        self.frozen_frame_fraction = Some(self.frame_fraction());
        self.missed_updates = None;
    }

    pub fn game_over(&mut self) {
        self.game_state = GameState::GameOver;
        self.frozen_frame_fraction = Some(self.frame_fraction());
    }

    pub fn graphics_frame_num(&self) -> usize {
        self.graphics_frame_num
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
