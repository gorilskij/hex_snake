use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use enum_rotate::EnumRotate;

use crate::rendering;
use crate::support::storage;

/// Bumped when a stored preference changes meaning rather than merely coming or
/// going — adding and removing settings is already handled by reading key by
/// key. Nothing migrates yet; this is here so that an old file can be
/// recognised as old when something eventually does.
const VERSION: u32 = 1;

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
    /// Where the head would come out if it teleported: the exit edges thicken
    /// and tint, the closer the head is to being able to use them
    Teleport,
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

/// The names each enum goes by in the stored file. These live here rather than
/// on the enums because they are a file-format concern: renaming a variant
/// should not silently invalidate everyone's saved preferences.
mod names {
    use super::{rendering, DrawGrid, HintStyle};

    pub fn draw_grid(value: DrawGrid) -> &'static str {
        match value {
            DrawGrid::Grid => "grid",
            DrawGrid::Dots => "dots",
            DrawGrid::None => "none",
        }
    }

    pub fn read_draw_grid(text: &str) -> Option<DrawGrid> {
        match text {
            "grid" => Some(DrawGrid::Grid),
            "dots" => Some(DrawGrid::Dots),
            "none" => Some(DrawGrid::None),
            _ => None,
        }
    }

    pub fn hint_style(value: HintStyle) -> &'static str {
        match value {
            HintStyle::Border => "border",
            HintStyle::Gradient => "gradient",
            HintStyle::Teleport => "teleport",
            HintStyle::None => "none",
        }
    }

    pub fn read_hint_style(text: &str) -> Option<HintStyle> {
        match text {
            "border" => Some(HintStyle::Border),
            "gradient" => Some(HintStyle::Gradient),
            "teleport" => Some(HintStyle::Teleport),
            "none" => Some(HintStyle::None),
            _ => None,
        }
    }

    pub fn draw_style(value: rendering::Style) -> &'static str {
        match value {
            rendering::Style::Hexagon => "hexagon",
            rendering::Style::Smooth => "smooth",
        }
    }

    pub fn read_draw_style(text: &str) -> Option<rendering::Style> {
        match text {
            "hexagon" => Some(rendering::Style::Hexagon),
            "smooth" => Some(rendering::Style::Smooth),
            _ => None,
        }
    }
}

/// Split `key=value` lines into a map. Anything without a `=` is skipped, so a
/// hand-edited file with a stray blank line or comment still reads.
fn parse(text: &str) -> HashMap<&str, &str> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim(), value.trim()))
        .collect()
}

/// Persisting preferences.
///
/// Only the settings that belong to the player are stored — the game-rule
/// fields alongside them (`apple_food`, `special_apples`, the spawn
/// probabilities) are not preferences and are due to move elsewhere.
///
/// Reading is done key by key against [`Prefs::default`], which is what makes
/// an old file keep working: a key that has since been removed is ignored, one
/// that did not exist yet keeps its default, and a value that no longer parses
/// costs only that one setting instead of the whole file.
impl Prefs {
    pub fn load() -> Self {
        let Some(text) = storage::load() else {
            return Self::default();
        };
        Self::read(&text)
    }

    pub fn save(&self) {
        storage::save(&self.to_text());
    }

    /// The stored form: one `key=value` per line.
    fn to_text(&self) -> String {
        [
            ("version", VERSION.to_string()),
            ("draw_grid", names::draw_grid(self.draw_grid).to_string()),
            ("draw_border", self.draw_border.to_string()),
            ("draw_distance_grid", self.draw_distance_grid.to_string()),
            ("draw_player_path", self.draw_player_path.to_string()),
            ("hint_style", names::hint_style(self.hint_style).to_string()),
            ("display_fps", self.display_fps.to_string()),
            ("display_stats", self.display_stats.to_string()),
            ("message_duration_ms", self.message_duration.as_millis().to_string()),
            ("draw_style", names::draw_style(self.draw_style).to_string()),
            ("hide_cursor", self.hide_cursor.to_string()),
        ]
        .iter()
        .map(|(key, value)| format!("{key}={value}\n"))
        .collect()
    }

    fn read(text: &str) -> Self {
        let fields = parse(text);
        let mut prefs = Self::default();

        let field = |key: &str| fields.get(key).copied();
        let flag = |key: &str, current: bool| field(key).and_then(|v| bool::from_str(v).ok()).unwrap_or(current);

        prefs.draw_grid = field("draw_grid")
            .and_then(names::read_draw_grid)
            .unwrap_or(prefs.draw_grid);
        prefs.hint_style = field("hint_style")
            .and_then(names::read_hint_style)
            .unwrap_or(prefs.hint_style);
        prefs.draw_style = field("draw_style")
            .and_then(names::read_draw_style)
            .unwrap_or(prefs.draw_style);

        prefs.draw_border = flag("draw_border", prefs.draw_border);
        prefs.draw_distance_grid = flag("draw_distance_grid", prefs.draw_distance_grid);
        prefs.draw_player_path = flag("draw_player_path", prefs.draw_player_path);
        prefs.display_fps = flag("display_fps", prefs.display_fps);
        prefs.display_stats = flag("display_stats", prefs.display_stats);
        prefs.hide_cursor = flag("hide_cursor", prefs.hide_cursor);

        prefs.message_duration = field("message_duration_ms")
            .and_then(|v| u64::from_str(v).ok())
            .map(Duration::from_millis)
            .unwrap_or(prefs.message_duration);

        prefs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_saved_file_reads_back_the_same() {
        let mut prefs = Prefs::default();
        prefs.draw_grid = DrawGrid::Dots;
        prefs.hint_style = HintStyle::Gradient;
        prefs.draw_style = rendering::Style::Hexagon;
        prefs.draw_border = false;
        prefs.display_fps = true;
        prefs.message_duration = Duration::from_millis(4500);

        assert_eq!(Prefs::read(&prefs.to_text()).to_text(), prefs.to_text());
    }

    /// A file from an older or newer build still loads: keys this build does
    /// not know are ignored, and ones it expects but cannot find keep their
    /// default.
    #[test]
    fn unknown_and_missing_keys_are_survivable() {
        let prefs = Prefs::read("version=1\ndraw_border=false\na_setting_from_the_future=42\n");

        assert!(!prefs.draw_border, "the key that was there applied");
        assert_eq!(
            prefs.display_stats,
            Prefs::default().display_stats,
            "the key that was missing kept its default",
        );
    }

    /// One unparseable value costs one setting rather than the whole file —
    /// the reason for reading key by key instead of deserializing in one go.
    #[test]
    fn a_bad_value_costs_only_itself() {
        let prefs = Prefs::read("draw_border=perhaps\ndisplay_fps=true\ndraw_grid=spirals\n");

        assert_eq!(prefs.draw_border, Prefs::default().draw_border, "fell back");
        assert!(matches!(prefs.draw_grid, DrawGrid::Grid), "fell back");
        assert!(prefs.display_fps, "but the good value beside it still applied");
    }

    /// Blank lines and anything without a `=` are skipped, so the file stays
    /// safe to edit by hand.
    #[test]
    fn stray_lines_are_ignored() {
        assert!(!Prefs::read("\n# a comment\n  draw_border = false  \n\ngarbage\n").draw_border);
    }

    /// The game-rule fields share the struct for now but are not preferences,
    /// so they must stay out of what gets written.
    #[test]
    fn rules_are_not_part_of_the_stored_form() {
        let mut prefs = Prefs::default();
        let before = prefs.to_text();

        prefs.apple_food = 5.;
        prefs.special_apples = !prefs.special_apples;
        prefs.prob_spawn_killer = 0.5;

        assert_eq!(prefs.to_text(), before);
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
