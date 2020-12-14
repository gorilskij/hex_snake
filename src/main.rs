#[macro_use]
extern crate derive_more;

use ggez::{
    event::run,
    ContextBuilder,
};
use crate::app::App;

mod app;

// TODO
//  two snakes's heads can pass through each other if iterated at the same time,
//  this seems like a bit of a fundamental problem (can't move 1/2 a cell)

fn main() {
    let app = &mut App::new();

    let (ctx, event_loop) = &mut ContextBuilder::new("hex_snake", "gorilskij")
        .window_mode(app.wm())
        .window_setup(app.ws())
        .build()
        .unwrap();

    // use CtrlLayout::*;
    // use CtrlSide::*;
    // let players = vec![
    //     // uncomment for 2-player
    //     // Ctrl {
    //     //     layout: Dvorak,
    //     //     keyboard_side: LeftSide,
    //     //     hand: RightSide,
    //     // },
    //
    //     Ctrl {
    //         layout: Dvorak,
    //         keyboard_side: RightSide,
    //         hand: RightSide,
    //     },
    // ];
    // let mut game = Game::new(10., players, Theme::default_dark(), wm);

    run(ctx, event_loop, app).expect("crashed")
}
