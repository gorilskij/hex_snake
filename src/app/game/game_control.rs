use crate::app::Frames;
use std::{
    cmp::max,
    collections::VecDeque,
    time::{Duration, Instant},
};

struct FPSCounter {
    step: Frames, // record once every 'step' frames
    n: Frames,    // counts down to recording the next frame
    buffer: VecDeque<Instant>,
}

impl FPSCounter {
    const LEN: usize = 10; // number of frame batches to store

    fn new(fps: u64) -> Self {
        let mut counter = Self {
            step: 0,
            n: 0,
            buffer: VecDeque::with_capacity(Self::LEN),
        };
        counter.set_fps(fps);
        counter
    }

    fn set_fps(&mut self, fps: Frames) {
        self.step = max(1, 3 * fps / Self::LEN as Frames);
        println!(
            "FPSCounter: fps = {}, step = {}, len = {}, frames = {}",
            fps,
            self.step,
            Self::LEN,
            self.step * Self::LEN as Frames
        );
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
            ((self.buffer.len() - 1) as Frames * self.step) as f64
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
    game_fps: u64,
    game_frame_duration: Duration,
    last_update: Instant,
    surplus: f64, // secs

    // counting down for the while loop in update()
    queued_updates: Option<usize>,

    graphics_frame_num: usize,

    measured_game_fps: FPSCounter,
    measured_graphics_fps: FPSCounter,

    game_state: GameState,
    frozen_frame_fraction: Option<f32>,
}

impl GameControl {
    pub fn new(fps: u64) -> Self {
        Self {
            game_fps: fps,
            game_frame_duration: Duration::from_nanos(1_000_000_000 / fps),
            last_update: Instant::now(),
            surplus: 0.,

            queued_updates: None,

            graphics_frame_num: 0,

            measured_game_fps: FPSCounter::new(fps),
            measured_graphics_fps: FPSCounter::new(60),

            game_state: GameState::Playing,
            frozen_frame_fraction: None,
        }
    }

    pub fn fps(&self) -> u64 {
        self.game_fps
    }

    pub fn set_fps(&mut self, fps: u64) {
        self.game_fps = fps;
        self.game_frame_duration = Duration::from_nanos(1_000_000_000 / fps);
        self.measured_game_fps.set_fps(fps);
    }

    // WARNING this will perform as many updates as the framerate requires
    //  this can cause strong lag a high framerates
    // TODO lower game framerate to keep up graphics framerate
    // call in update(), run update code this many times
    pub fn can_update(&mut self) -> bool {
        if self.game_state != GameState::Playing {
            return false;
        }

        match &mut self.queued_updates {
            Some(0) => {
                self.queued_updates = None;
                false
            }
            Some(n) => {
                *n -= 1;
                true
            }
            None => {
                let game_frames = self.last_update.elapsed().as_secs_f64()
                    / self.game_frame_duration.as_secs_f64()
                    + self.surplus;
                let updates = game_frames as usize;

                if updates > 0 {
                    self.surplus = game_frames % 1.;
                    self.last_update = Instant::now();

                    self.queued_updates = Some(updates - 1);

                    // TODO: O(1)ize
                    for _ in 0..updates {
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
            Some(frac) => {
                let mut elapsed =
                    (frac - self.surplus as f32) * self.game_frame_duration.as_secs_f32();
                // slight tolerance
                if (-0.01..0.).contains(&elapsed) {
                    elapsed = 0.;
                } else {
                    assert!(elapsed >= 0., "elapsed: {}s", elapsed);
                }
                self.last_update = Instant::now() - Duration::from_secs_f32(elapsed);
            }
        }
    }

    pub fn pause(&mut self) {
        self.game_state = GameState::Paused;
        self.frozen_frame_fraction = Some(self.frame_fraction());
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
                    + self.surplus as f32;
                if frac > 1. {
                    1.
                } else {
                    frac
                }
            }
        }
    }

    pub fn measured_game_fps(&self) -> f64 {
        self.measured_game_fps.fps()
    }

    pub fn measured_graphics_fps(&self) -> f64 {
        self.measured_graphics_fps.fps()
    }
}
