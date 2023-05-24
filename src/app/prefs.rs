use crate::basic::Food;
use crate::rendering;
use std::time::Duration;

#[derive(Copy, Clone)]
pub enum DrawGrid {
    Grid,
    Dots,
    None,
}

impl DrawGrid {
    // TODO: turn into a trait and write a derive macro for it
    // TODO: rename
    pub fn rotate(&mut self) -> Self {
        *self = match self {
            DrawGrid::Grid => DrawGrid::Dots,
            DrawGrid::Dots => DrawGrid::None,
            DrawGrid::None => DrawGrid::Grid,
        };
        *self
    }
}

pub struct Prefs {
    pub draw_grid: DrawGrid,
    pub draw_border: bool,
    pub draw_distance_grid: bool,
    pub draw_player_path: bool,

    pub display_fps: bool,
    pub display_stats: bool,
    pub message_duration: Duration,

    pub apple_food: Food,
    pub special_apples: bool,
    pub prob_spawn_competitor: f64,
    pub prob_spawn_killer: f64,
    pub prob_spawn_rain: f64,

    pub draw_style: rendering::Style,
    // pub draw_ai_debug_artifacts: bool,

    pub hide_cursor: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            draw_grid: DrawGrid::None,
            draw_border: false,
            draw_distance_grid: false,
            draw_player_path: false,

            display_fps: false,
            display_stats: false,
            message_duration: Duration::from_secs(2),

            apple_food: 1,
            special_apples: true,
            prob_spawn_competitor: 0.025,
            prob_spawn_killer: 0.015,
            prob_spawn_rain: 0.002,

            draw_style: rendering::Style::Smooth,
            // draw_ai_debug_artifacts: false,

            hide_cursor: true,
        }
    }
}

// builder
impl Prefs {
    pub fn apple_food(mut self, food: Food) -> Self {
        self.apple_food = food;
        self
    }

    pub fn special_apples(mut self, special_apples: bool) -> Self {
        self.special_apples = special_apples;
        self
    }
}
