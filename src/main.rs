#[macro_use]
extern crate derive_more;

use crate::game::{theme::Theme, Game};
use ggez::{
    conf::{FullscreenType, NumSamples, WindowMode, WindowSetup},
    event::run,
    ContextBuilder,
};

mod game;

// error
// NOTE it's still possible to eat yourself by going back, input queue needed!
//  also, two snakes's heads can pass through each other if iterated at the same time,
//  this seems like a bit of a fundamental problem (can't move 1/2 a cell)
// todo fix

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

    let (ctx, event_loop) = &mut ContextBuilder::new("game", "author")
        .window_mode(wm)
        .window_setup(ws)
        .build()
        .unwrap();

    let players = vec![
        ctrl! {
            layout: qwerty,
            side: right,
            hand: right,
        },
        // uncomment for 2-player
        // ctrl! {
        //     layout: qwerty,
        //     side: left,
        //     hand: right,
        // },
    ];
    let mut game = Game::new(10., players, Theme::DEFAULT_DARK, wm);

    run(ctx, event_loop, &mut game).expect("crashed")
}
