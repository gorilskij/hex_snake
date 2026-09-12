use std::time::Duration;

use enum_rotate::EnumRotate;

use crate::rendering;

#[derive(Copy, Clone, EnumRotate)]
pub enum DrawGrid {
    Grid,
    Dots,
    None,
}

/// How wrap-around edges hint at what lies on the other side (see
/// `app::border_hints`).
#[derive(Copy, Clone, Eq, PartialEq, EnumRotate)]
pub enum HintStyle {
    /// Recolor the edge's stretch of the border
    Border,
    /// A gradient fading from the edge into its cell
    Gradient,
    None,
}

pub struct Prefs {
    pub draw_grid: DrawGrid,
    pub draw_border: bool,
    pub draw_distance_grid: bool,
    pub draw_player_path: bool,
    pub hint_style: HintStyle,

    pub display_fps: bool,
    pub display_stats: bool,
    pub message_duration: Duration,

    pub apple_food: f32,
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
            draw_grid: DrawGrid::Grid,
            draw_border: true,
            draw_distance_grid: false,
            draw_player_path: false,
            hint_style: HintStyle::Border,

            display_fps: false,
            display_stats: false,
            message_duration: Duration::from_secs(2),

            apple_food: 1.,
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
    pub fn apple_food(mut self, food: f32) -> Self {
        self.apple_food = food;
        self
    }

    pub fn special_apples(mut self, special_apples: bool) -> Self {
        self.special_apples = special_apples;
        self
    }
}
