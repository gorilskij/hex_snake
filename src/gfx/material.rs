//! Custom shader material for snake coloring.
//!
//! Instead of chopping each segment into many flat-colored subsegments (the old
//! CPU approach), snake geometry carries a per-vertex "position along the body"
//! in `uv.x`, and this fragment shader samples a 1-D **palette lookup texture**
//! (LUT) by that coordinate. The gradient follows the body through turns because
//! the coordinate is baked into the (body-shaped) geometry; it is smooth because
//! the LUT is sampled with linear filtering.
//!
//! macOS defaults to OpenGL and the web target is WebGL, so a single GLSL-ES 100
//! shader covers both — no Metal variant required.

use macroquad::material::{load_material, Material, MaterialParams};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, PipelineParams, ShaderSource};
use macroquad::texture::{FilterMode, Texture2D};

use crate::gfx::graphics::Color;
use crate::gfx::{GameError, GameResult};

const VERTEX: &str = r#"#version 100
precision highp float;
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 normal;
varying vec2 uv;
varying vec2 seg_bounds;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
    // normal.xy carries this segment's [lo, hi] uv range (see push_shaded_ribbon)
    seg_bounds = normal.xy;
}"#;

// Samples the palette LUT by uv.x ("distance along body"). The LUT is a
// single-row texture, so v is fixed at 0.5. uv.y (across-width) is currently
// ignored, matching the old flat-across-width look; it is available for future
// shading.
//
// The lookup is clamped to this segment's own uv range (already inset by half a
// texel on the CPU, see build_shaded) so linear filtering never bleeds across a
// segment boundary: each segment samples only its own LUT slot, so hard color
// steps land exactly on the geometry seam while gradients within a segment stay
// smooth.
const FRAGMENT: &str = r#"#version 100
precision highp float;
varying vec2 uv;
varying vec2 seg_bounds;
uniform sampler2D Texture;
void main() {
    float u = clamp(uv.x, seg_bounds.x, seg_bounds.y);
    gl_FragColor = texture2D(Texture, vec2(u, 0.5));
}"#;

pub fn snake_material() -> GameResult<Material> {
    load_material(
        ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
        MaterialParams {
            // Match the default alpha-over blending the standard mesh path
            // uses, so semi-transparent segments composite the same way.
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .map_err(|e| GameError(format!("failed to load snake material: {e:?}")))
}

/// A 1-D palette lookup texture: one row of RGBA texels, sampled by body
/// fraction in `[0, 1]`. Linear filtering makes the gradient continuous.
pub struct PaletteLut {
    texture: Texture2D,
}

impl PaletteLut {
    pub fn new(colors: &[Color]) -> Self {
        let texture = Texture2D::from_rgba8(colors.len() as u16, 1, &to_rgba8(colors));
        // Linear keeps within-segment gradients smooth. The shader clamps each
        // segment to its own LUT slot, so hard steps between segments stay sharp
        // and stable regardless of this filtering.
        texture.set_filter(FilterMode::Linear);
        Self { texture }
    }

    /// The underlying texture (cheap clone — it is a GPU handle) to hang on a
    /// mesh's `texture` field so the shader's `Texture` sampler resolves to it.
    pub fn texture(&self) -> Texture2D {
        self.texture.clone()
    }
}

fn to_rgba8(colors: &[Color]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(colors.len() * 4);
    for c in colors {
        bytes.push((c.r.clamp(0., 1.) * 255.) as u8);
        bytes.push((c.g.clamp(0., 1.) * 255.) as u8);
        bytes.push((c.b.clamp(0., 1.) * 255.) as u8);
        bytes.push((c.a.clamp(0., 1.) * 255.) as u8);
    }
    bytes
}
