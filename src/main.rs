#![feature(stmt_expr_attributes)]
#![feature(if_let_guard)]
#![feature(try_blocks)]
#![deny(unused_must_use)]
// #![deny(unsafe_code)]

#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate lazy_static;

use ggez::{event::run, ContextBuilder};

use crate::{
    app::{keyboard_control::ControlSetup, App},
    basic::Side,
    keyboard::Layout,
};
use ggez::conf::{FullscreenType, NumSamples, WindowMode, WindowSetup};

#[macro_use]
mod basic;
mod app;
mod color;
mod keyboard;
mod partial_min_max;
mod row;
mod support;

// TODO
//  - untie frame_fraction from graphics
//  - factor out lazy redrawing code (the whole mess with Some(grid_mesh)...)
//  - if a frame is nearing its end, delay turning until the next frame to avoid choppy animation
//  - make border toggleable
//  - smooth animation when cutting
//  - diagnose high cpu use when paused

// TODO
//  implement snake rain, snakes fall from the sky and flow along other snakes, it would look awesome

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

    let (mut ctx, event_loop) = ContextBuilder::new("hex_snake", "gorilskij")
        .window_mode(window_mode)
        .window_setup(window_setup)
        .build()
        .unwrap();

    let app = App::new(
        vec![ControlSetup {
            layout: Layout::Dvorak,
            keyboard_side: Side::Right,
            hand: Side::Right,
        }],
        &mut ctx,
    );

    println!("start");

    // ggez::graphics::set_screen_coordinates(&mut ctx, Rect { x: 0., y: 0., w: width, h: height })
    //     .unwrap();

    run(ctx, event_loop, app)
}
