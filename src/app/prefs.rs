use crate::app::{rendering, utils::Food};
use std::time::Duration;

pub struct Prefs {
    pub draw_grid: bool,
    pub draw_border: bool,

    pub display_fps: bool,
    pub display_stats: bool,
    pub message_duration: Duration,

    pub apple_food: Food,
    pub special_apples: bool,
    pub prob_spawn_competitor: f64,
    pub prob_spawn_killer: f64,
    pub prob_spawn_rain: f64,

    pub draw_style: rendering::Style,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            draw_grid: true,
            draw_border: true,

            display_fps: false,
            display_stats: false,
            message_duration: Duration::from_secs(2),

            apple_food: 1,
            special_apples: true,
            prob_spawn_competitor: 0.025,
            prob_spawn_killer: 0.015,
            prob_spawn_rain: 0.002,

            draw_style: rendering::Style::Smooth,
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
