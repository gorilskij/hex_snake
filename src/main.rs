#![deny(unused_must_use)]
#![feature(stmt_expr_attributes)] // for rustfmt::skip on expressions
#![feature(if_let_guard)]

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
use ggez::{
    conf::{FullscreenType, NumSamples, WindowMode, WindowSetup},
};

mod app;
mod basic;
mod color;
mod partial_min_max;
mod row;

fn main() {
    let width = 2000.;
    let height = 1600.;

    let window_mode = WindowMode {
        width,
        height,
        maximized: false,
        fullscreen_type: FullscreenType::Windowed,
        borderless: false,
        min_width: 0.,
        min_height: 0.,
        max_width: 0.,
        max_height: 0.,
        resizable: true,
        visible: true,
        resize_on_scale_factor_change: false,
    };

    let window_setup = WindowSetup {
        title: "Hex Snake".to_string(),
        samples: NumSamples::One,
        vsync: true,
        icon: "".to_string(),
        srgb: true,
    };

    let app = App::new(
        vec![ControlSetup {
            layout: KeyboardLayout::Dvorak,
            keyboard_side: Side::Right,
            hand: Side::Right,
        }],
        window_mode,
        window_setup,
    );

    let (mut ctx, event_loop) = ContextBuilder::new("hex_snake", "gorilskij")
        .window_mode(app.wm())
        .window_setup(app.ws())
        .build()
        .unwrap();

    // ggez::graphics::set_screen_coordinates(&mut ctx, Rect { x: 0.0, y: 0.0, w: width, h: height })
    //     .unwrap();

    run(ctx, event_loop, app)
}
