use crate::app::message;
use crate::app::message::Message;

/// Collect statistics about the current game state
#[derive(Default)]
pub struct Stats {
    /// Number of polygons built this frame (only meshes that were rebuilt)
    pub polygons: usize,
    /// The player snake's true length in cells
    pub player_length: Option<f32>,
}

impl Stats {
    pub fn get_stats_message(&self) -> Message {
        let length = match self.player_length {
            Some(length) => format!("{length:.3}"),
            None => "-".to_string(),
        };
        let text = format!("polygons: {}\nsnake length: {length}", self.polygons);
        Message {
            text,
            position: message::Position::TopLeft,
            h_margin: Message::DEFAULT_MARGIN,
            v_margin: Message::DEFAULT_MARGIN * 2. + Message::DEFAULT_FONT_SIZE,
            font_size: Message::DEFAULT_FONT_SIZE,
            color: crate::color::WHITE,
            disappear: None,
            background: true,
        }
    }
}
