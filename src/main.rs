#[macro_use] extern crate derive_more;

use ggez::conf::{WindowMode, FullscreenType};
use ggez::ContextBuilder;
use ggez::event::run;
use crate::game::Game;

mod game;

fn main() {
    let wm = WindowMode {
        width: 1000.,
        height: 800.,
        maximized: false,
        fullscreen_type: FullscreenType::Windowed,
        borderless: false,
        min_width: 0.,
        min_height: 0.,
        max_width: 0.,
        max_height: 0.,
        resizable: true,
    };

    let (ref mut ctx, ref mut event_loop)
        = ContextBuilder::new("game", "author")
        .window_mode(wm)
        .build()
        .unwrap();

    let mut game = Game::new(10., wm);
    run(ctx, event_loop, &mut game).expect("crashed")
}
