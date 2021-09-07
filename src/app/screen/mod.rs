pub use debug_scenario::DebugScenario;
pub use game::Game;
pub use start_screen::StartScreen;

pub use crate::app::apple::{Apple, Type};
pub use crate::app::prefs::{ Prefs};

mod game;
mod rendering;
mod start_screen;
mod debug_scenario;

pub enum Screen {
    DebugScenario(DebugScenario),
    StartScreen(StartScreen),
    Game(Game),
}
