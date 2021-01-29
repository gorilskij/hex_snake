use std::{
    cmp::max,
    collections::VecDeque,
    time::{Duration, Instant},
};

struct FPSCounter {
    len: usize,
    buffer: VecDeque<Instant>,
}

impl FPSCounter {
    fn new(fps: u64) -> Self {
        // TODO: calculate required len in a way that makes sense
        //  maybe also register every n frames for higher fps to avoid wasting space
        let len = max(60, fps as usize);
        Self {
            len,
            buffer: VecDeque::with_capacity(len),
        }
    }

    fn set_fps(&mut self, fps: u64) {
        self.len = max(60, fps as usize);
        self.reset();
    }

    fn register_frame(&mut self) {
        if self.buffer.len() >= self.len {
            self.buffer.pop_front();
        }
        self.buffer.push_back(Instant::now());
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }

    fn fps(&self) -> f64 {
        if self.buffer.len() >= 2 {
            (self.buffer.len() - 1) as f64
                / (self.buffer[self.buffer.len() - 1] - self.buffer[0]).as_secs_f64()
        } else {
            0.
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
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

    frame_num: usize,

    measured_game_fps: FPSCounter,
    measured_graphics_fps: FPSCounter,

    game_state: GameState,
}

impl GameControl {
    pub fn new(fps: u64) -> Self {
        Self {
            game_fps: fps,
            game_frame_duration: Duration::from_nanos(1_000_000_000 / fps),
            last_update: Instant::now(),
            surplus: 0.,

            frame_num: 0,

            measured_game_fps: FPSCounter::new(fps),
            measured_graphics_fps: FPSCounter::new(60),

            game_state: GameState::Playing,
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
    pub fn num_updates(&mut self) -> usize {
        let game_frames = self.last_update.elapsed().as_secs_f64()
            / self.game_frame_duration.as_secs_f64()
            + self.surplus;
        let updates = game_frames as usize;

        if updates > 0 {
            self.surplus = game_frames % 1.;
            self.last_update = Instant::now();
            self.frame_num = self.frame_num.wrapping_add(updates);
        }

        if self.game_state == GameState::Playing {
            for _ in 0..updates {
                self.measured_game_fps.register_frame();
            }
            updates
        } else {
            0
        }
    }

    // call in draw()
    pub fn graphics_frame(&mut self) {
        self.measured_graphics_fps.register_frame();
    }

    pub fn state(&self) -> GameState {
        self.game_state
    }

    pub fn play(&mut self) {
        self.game_state = GameState::Playing;
        self.measured_game_fps.reset();
    }

    pub fn pause(&mut self) {
        self.game_state = GameState::Paused;
    }

    pub fn game_over(&mut self) {
        self.game_state = GameState::GameOver;
    }

    pub fn frame_num(&self) -> usize {
        self.frame_num
    }

    // fraction of the current game frame that has elapsed
    pub fn frame_fraction(&self) -> f32 {
        let game_frames = self.last_update.elapsed().as_secs_f32()
            / self.game_frame_duration.as_secs_f32()
            + self.surplus as f32;
        if game_frames > 1. {
            1.
        } else {
            game_frames
        }
    }

    pub fn measured_game_fps(&self) -> f64 {
        self.measured_game_fps.fps()
    }

    pub fn measured_graphics_fps(&self) -> f64 {
        self.measured_graphics_fps.fps()
    }
}