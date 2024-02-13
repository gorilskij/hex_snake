#![feature(stmt_expr_attributes)]
#![feature(if_let_guard)]
#![feature(try_blocks)]
#![feature(never_type)]
#![feature(exhaustive_patterns)]
#![deny(unused_must_use)]
// #![deny(unsafe_code)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]
#![feature(trace_macros)]
// #![feature(return_position_impl_trait_in_trait)]

#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate lazy_static;
extern crate core;

use ggez::event::run;
use ggez::ContextBuilder;

use crate::app::keyboard_control::ControlSetup;
use crate::app::App;
use crate::basic::Side;
use crate::keyboard_layout::Layout;
use ggez::conf::{FullscreenType, NumSamples, WindowMode, WindowSetup};

#[macro_use]
mod support;
#[macro_use]
mod basic;
mod app;
mod color;
mod keyboard_layout;
mod snake;
mod view;
#[macro_use]
mod apple;
mod button;
mod error;
mod rendering;
pub mod snake_control;

// TODO: upgrading to ggez 0.8 made the colors duller, fix that

// TODO
//  - untie frame_fraction from graphics
//  - factor out lazy redrawing code (the whole mess with Some(grid_mesh)...)
//  - smooth animation when cutting
//  - diagnose high cpu use when paused

// TODO
//  make rain snakes ignore food
//  make head-to-head collision with rain also ignore

fn main() {
    let width = 2000.;
    let height = 1600.;

    let window_mode = WindowMode {
        width,
        height,
        maximized: false,
        fullscreen_type: FullscreenType::Windowed,
        borderless: false,
        transparent: false,
        min_width: 1.,
        min_height: 1.,
        max_width: 0.,
        max_height: 0.,
        resizable: true,
        visible: true,
        resize_on_scale_factor_change: false,
        logical_size: None,
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
