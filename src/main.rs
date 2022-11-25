#![feature(stmt_expr_attributes)]
#![feature(if_let_guard)]
#![feature(try_blocks)]
// #![feature(never_type)]
#![deny(unused_must_use)]
// #![deny(unsafe_code)]
// #![feature(trace_macros)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]
// #![feature(return_position_impl_trait_in_trait)]

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
mod support;
#[macro_use]
mod basic;
mod app;
mod color;
mod keyboard;
mod pathfinding;
mod snake;
mod view;
#[macro_use]
mod apple;
mod error;
mod rendering;
pub mod snake_control;

// TODO
//  - untie frame_fraction from graphics
//  - factor out lazy redrawing code (the whole mess with Some(grid_mesh)...)
//  - if a frame is nearing its end, delay turning until the next frame to avoid choppy animation
//  - make border toggleable
//  - smooth animation when cutting
//  - diagnose high cpu use when paused

// TODO
//  make rain snakes ignore food
//  split EatBehavior::Ignore into "pass in front" and "pass behind"
//   use "pass behind for rain"
//  make head-to-head collision with rain also ignore
//  allow snake to pass (tunnel) under is eaten segments

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
