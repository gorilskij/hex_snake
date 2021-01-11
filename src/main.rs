#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate lazy_static;

use ggez::{event::run, ContextBuilder};

use crate::app::App;

mod app;

// two snakes's heads can pass through each other if iterated at the same time,
// this seems like a bit of a fundamental problem (can't move 1/2 a cell)

fn main() {
    let app = &mut App::new();

    let (ctx, event_loop) = &mut ContextBuilder::new("hex_snake", "gorilskij")
        .window_mode(app.wm())
        .window_setup(app.ws())
        .build()
        .unwrap();

    run(ctx, event_loop, app).expect("crashed")
}
