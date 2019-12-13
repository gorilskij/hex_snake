use ggez::conf::{WindowMode, FullscreenType};
use ggez::{ContextBuilder, conf};
use ggez::event::run;
use crate::game::Game;

mod game;

const WIDTH: f32 = 1000.;
const HEIGHT: f32 = 800.;

fn main() {
    let wm = WindowMode {
        width: WIDTH,
        height: HEIGHT,
        maximized: false,
        fullscreen_type: FullscreenType::Windowed,
        borderless: false,
        min_width: 0.,
        min_height: 0.,
        max_width: 0.,
        max_height: 0.,
        resizable: false
    };

    let (ref mut ctx, ref mut event_loop)
        = ContextBuilder::new("game", "author")
        .window_mode(wm)
        .build()
        .unwrap();

    let mut game = Game::new(100, 80);
    run(ctx, event_loop, &mut game).expect("crashed")
}
