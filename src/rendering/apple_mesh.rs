use std::time::Duration;

use anyhow::Result;
use hsl::HSL;

use crate::app::game_context::GameContext;
use crate::app::stats::Stats;
use crate::apple::Apple;
use crate::color::to_color::ToColor;
use crate::rendering;
use crate::rendering::shape::{Hexagon, Shape};
use crate::support::mesh::{build_circle, build_polygon, DrawMode, Mesh};

pub fn apple_mesh(apples: &[Apple], gtx: &GameContext, elapsed_total: Duration, stats: &mut Stats) -> Result<Mesh> {
    assert!(!apples.is_empty(), "tried to draw a mesh with 0 apples");

    stats.redrawing_apples = true;

    let mut parts: Vec<Mesh> = Vec::with_capacity(apples.len());

    for apple in apples {
        use crate::apple::Type::*;
        let color = match apple.apple_type {
            Eat(_) => gtx.palette.apple_color,
            Shrink(_) => gtx.palette.apple_color, // TODO: change
            SpawnSnake(_) | SpawnRain => {
                let hue = 360. * (elapsed_total.as_millis() as f64 / 1000. % 1.);
                let hsl = HSL { h: hue, s: 1., l: 0.3 };
                hsl.to_color()
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
