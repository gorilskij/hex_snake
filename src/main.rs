#[macro_use] extern crate derive_more;

use ggez::conf::{WindowMode, FullscreenType, WindowSetup, NumSamples};
use ggez::ContextBuilder;
use ggez::event::run;
use crate::game::Game;
use crate::game::theme::Theme;

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
        resizable: false,
    };
    
    let ws = WindowSetup {
        title: "Hex Snake".to_string(),
        samples: NumSamples::Zero,
        vsync: true,
        icon: "".to_string(),
        srgb: true,
    };

    let (ctx, event_loop)
        = &mut ContextBuilder::new("game", "author")
        .window_mode(wm)
        .window_setup(ws)
        .build()
        .unwrap();

    let mut game = Game::new(10., Theme::DEFAULT_DARK, wm);
    run(ctx, event_loop, &mut game).expect("crashed")
}