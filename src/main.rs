#![deny(unused_must_use)]
#![feature(stmt_expr_attributes)] // for rustfmt::skip on expressions

#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate lazy_static;

use ggez::{event::run, ContextBuilder};

use crate::{
    app::{
        keyboard_control::{ControlSetup, KeyboardLayout},
        App,
    },
    basic::Side,
};

mod app;
mod basic;
mod oklab;

fn main() {
    // NOTE: configure number of players and controls here
    let app = &mut App::new(vec![ControlSetup {
        layout: KeyboardLayout::Qwerty,
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
