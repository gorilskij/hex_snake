use crate::basic::DrawStyle;
use std::time::Duration;

pub type Food = u32;

pub struct Prefs {
    pub draw_grid: bool,
    pub draw_border: bool,
    pub display_fps: bool,
    pub display_stats: bool,
    pub apple_food: Food,
    pub message_duration: Duration,
    pub draw_style: DrawStyle,
    pub special_apples: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            draw_grid: true,
            draw_border: true,
            display_fps: false,
            display_stats: false,
            apple_food: 1,
            message_duration: Duration::from_secs(2),
            draw_style: DrawStyle::Smooth,
            special_apples: true,
        }
    }
}
