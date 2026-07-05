use hsl::HSL;

use crate::app::fps_control::FpsContext;
use crate::app::game_context::GameContext;
use crate::app::stats::Stats;
use crate::apple::Apple;
use anyhow::Result;
use crate::gfx::graphics::{build_circle, build_polygon, Color, DrawMode, Mesh};
use crate::rendering;
use crate::rendering::shape::{Hexagon, Shape};

pub fn apple_mesh(apples: &[Apple], gtx: &GameContext, ftx: &FpsContext, stats: &mut Stats) -> Result<Mesh> {
    assert!(!apples.is_empty(), "tried to draw a mesh with 0 apples");

    stats.redrawing_apples = true;

    let mut parts: Vec<Mesh> = Vec::with_capacity(apples.len());

    for apple in apples {
        use crate::apple::Type::*;
        let color = match apple.apple_type {
            Food(_) => gtx.palette.apple_color,
            SpawnSnake(_) | SpawnRain => {
                let hue = 360. * (ftx.elapsed_millis as f64 / 1000. % 1.);
                let hsl = HSL { h: hue, s: 1., l: 0.3 };
                Color::from(hsl.to_rgb())
            }
        };

        match gtx.prefs.draw_style {
            rendering::Style::Hexagon => {
                let dest = apple.pos.to_cartesian(gtx.cell_dim);
                let points = Hexagon::new(gtx.cell_dim).translate(dest);
                parts.push(build_polygon(DrawMode::fill(), &points, color));
            }
            rendering::Style::Smooth => {
                let dest = apple.pos.to_cartesian(gtx.cell_dim) + gtx.cell_dim.center();
                let radius = gtx.cell_dim.side / 2.;
                parts.push(build_circle(DrawMode::fill(), dest, radius, color));
            }
        }
        stats.polygons += 1;
    }

    Ok(Mesh::combine(parts))
}
