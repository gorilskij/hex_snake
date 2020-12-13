#[macro_use]
extern crate derive_more;

use crate::game::{theme::Theme, Game};
use ggez::{
    conf::{FullscreenType, NumSamples, WindowMode, WindowSetup},
    event::run,
    ContextBuilder,
};
use crate::game::ctrl::{Ctrl, CtrlLayout, CtrlSide};

mod game;

// TODO
//  two snakes's heads can pass through each other if iterated at the same time,
//  this seems like a bit of a fundamental problem (can't move 1/2 a cell)

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
        // resizable: false,
        resizable: true,
    };

    let ws = WindowSetup {
        title: "Hex Snake".to_string(),
        samples: NumSamples::Zero,
        vsync: true,
        icon: "".to_string(),
        srgb: true,
    };

    let (ctx, event_loop) = &mut ContextBuilder::new("hex_snake", "gorilskij")
        .window_mode(wm)
        .window_setup(ws)
        .build()
        .unwrap();

    use CtrlLayout::*;
    use CtrlSide::*;
    let players = vec![
        // uncomment for 2-player
        // Ctrl {
        //     layout: Dvorak,
        //     keyboard_side: LeftSide,
        //     hand: RightSide,
        // },

        Ctrl {
            layout: Dvorak,
            keyboard_side: RightSide,
            hand: RightSide,
        },
    ];
    let mut game = Game::new(10., players, Theme::default_dark(), wm);

    run(ctx, event_loop, &mut game).expect("crashed")
}
