#![deny(unused_must_use)]
#![feature(stmt_expr_attributes)]
#![feature(if_let_guard)]

#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate lazy_static;

use ggez::{event::run, ContextBuilder};

use crate::{
    app::{
        keyboard_control::{ControlSetup},
        App,
    },
    basic::Side,
};
use ggez::{
    conf::{FullscreenType, NumSamples, WindowMode, WindowSetup},
    graphics::Rect,
};
use crate::keyboard::Layout;

mod app;
mod basic;
mod color;
mod partial_min_max;
mod row;
mod keyboard;

// TODO
//  - if a frame is nearing its end, delay turning until the next frame to avoid choppy animation
//  - make border toggleable
//  - smooth animation when cutting
//  - diagnose high cpu use when paused

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
            layout: Layout::Dvorak,
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

    eprintln!("start");

    // ggez::graphics::set_screen_coordinates(&mut ctx, Rect { x: 0., y: 0., w: width, h: height })
    //     .unwrap();

    run(ctx, event_loop, app)
}
