pub use debug_scenario::DebugScenario;
pub use game::Game;
pub use start_screen::StartScreen;

pub use crate::app::prefs::Prefs;

mod debug_scenario;
mod game;
mod start_screen;

pub enum Screen {
    DebugScenario(DebugScenario),
    StartScreen(StartScreen),
    Game(Game),
}
