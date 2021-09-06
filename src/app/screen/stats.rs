use crate::app::screen::message::Message;
use ggez::graphics::Color;

/// Collect statistics about the current game state
#[derive(Default)]
pub struct Stats {
    /// Total number of graphical (sub)segments
    /// currently visible
    pub polygons: usize,
    /// Number of subsegments per segment (note
    /// that head and tail will have fewer).
    /// Maximum in the case of multiple snakes
    pub max_subsegments_per_segment: usize,
    pub redrawing_apples: bool,
    pub redrawing_snakes: bool,
}

impl Stats {
    pub fn get_stats_message(&self) -> Message {
        let text = format!(
            "total polygons: {}\nmax subsegments: {}\nredrawing apples: {}\nredrawing snakes: {}",
            self.polygons,
            self.max_subsegments_per_segment,
            self.redrawing_apples,
            self.redrawing_snakes,
        );
        Message {
            text,
            left: true,
            top: true,
            h_margin: Message::DEFAULT_MARGIN,
            v_margin: Message::DEFAULT_MARGIN * 2. + Message::DEFAULT_FONT_SIZE,
            font_size: Message::DEFAULT_FONT_SIZE,
            color: Color::WHITE,
            disappear: None,
        }
    }
}
