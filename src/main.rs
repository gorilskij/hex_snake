#![deny(unused_must_use)]

#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate lazy_static;

use ggez::{event::run, ContextBuilder};

use crate::app::{
    keyboard_control::{ControlSetup, KeyboardLayout, Side},
    App,
};

mod app;
mod times;

fn main() {
    // NOTE: configure number of players and controls here
    let app = &mut App::new(vec![ControlSetup {
        layout: KeyboardLayout::Dvorak,
        keyboard_side: Side::Right,
        hand: Side::Right,
    }]);

    let (ctx, event_loop) = &mut ContextBuilder::new("hex_snake", "gorilskij")
        .window_mode(app.wm())
        .window_setup(app.ws())
        .build()
        .unwrap();

    run(ctx, event_loop, app).expect("crashed")
}
