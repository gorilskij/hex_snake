use crate::app::message;
use crate::app::message::Message;
use crate::color::Color;

/// Collect statistics about the current game state
#[derive(Default)]
pub struct Stats {
    /// Total number of graphical (sub)segments
    /// currently visible
    pub polygons: usize,
    /// Number of subsegments per segment (note
    /// that head and tail will have fewer).
    /// Maximum in the case of multiple snakes
    pub max_color_resolution: usize,
    pub redrawing_apples: bool,
    pub redrawing_snakes: bool,
}

impl Stats {
    pub fn get_stats_message(&self) -> Message {
        let text = format!(
            "total polygons: {}\nmax subsegments: {}\nredrawing apples: {}\nredrawing snakes: {}",
            self.polygons, self.max_color_resolution, self.redrawing_apples, self.redrawing_snakes,
        );
        Message {
            text,
            position: message::Position::TopLeft,
            h_margin: Message::DEFAULT_MARGIN,
            v_margin: Message::DEFAULT_MARGIN * 2. + Message::DEFAULT_FONT_SIZE,
            font_size: Message::DEFAULT_FONT_SIZE,
            color: Color::WHITE,
            disappear: None,
        }
    }
}
