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

use anyhow::{Context, Result};
use macroquad::color::Color;
use macroquad::material::{load_material, Material, MaterialParams};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, PipelineParams, ShaderSource};
use macroquad::texture::{FilterMode, Texture2D};

const VERTEX: &str = r#"#version 100
precision highp float;
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 normal;
varying vec2 uv;
varying vec2 seg_bounds;
varying float brightness;
varying float along;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
    // normal.xy carries this segment's [lo, hi] uv range, normal.z the
    // brightness multiplier and normal.w how it is lit (see Surface)
    seg_bounds = normal.xy;
    brightness = normal.z;
    along = normal.w;
}"#;

// Samples the palette LUT by uv.x ("distance along body"). The LUT is a
// single-row texture, so v is fixed at 0.5.
//
// The lookup is clamped to this segment's own uv range (already inset by half a
// texel on the CPU, see build_shaded) so linear filtering never bleeds across a
// segment boundary: each segment samples only its own LUT slot, so hard color
// steps land exactly on the geometry seam while gradients within a segment stay
// smooth.
//
// The color is then scaled by the vertex's brightness (1 for the body itself;
// less for darker details drawn over it, like passability marks).
//
// Finally it is lit, unless the surface is flat (along < 0). The body is a
// round tube lit from straight above: uv.y runs across it, so x = 2·uv.y − 1 is
// where on the tube's cross-section the pixel is, and √(1 − x²) how much its
// surface faces up. The ends are hemispheres, like a pill's: along (0 where an
// end joins the body, 1 at its tip) tilts the surface away along the body too,
// so it faces up by √(1 − along²)·√(1 − x²). Facing up is all that counts with
// the light and the eye both straight above: diffuse light (Lambert) is that,
// and the specular highlight (Blinn-Phong) a power of it.
const FRAGMENT: &str = r#"#version 100
precision highp float;
varying vec2 uv;
varying vec2 seg_bounds;
varying float brightness;
varying float along;
uniform sampler2D Texture;
const float AMBIENT = 0.45;
const float DIFFUSE = 0.55;
const float SPECULAR = 0.3;
const float SHININESS = 24.0;
void main() {
    float u = clamp(uv.x, seg_bounds.x, seg_bounds.y);
    vec4 color = texture2D(Texture, vec2(u, 0.5));
    vec3 base = color.rgb * brightness;
    if (along < -0.5) {
        gl_FragColor = vec4(base, color.a);
        return;
    }
    float x = uv.y * 2.0 - 1.0;
    float up = sqrt(max(0.0, 1.0 - x * x)) * sqrt(max(0.0, 1.0 - along * along));
    vec3 lit = base * (AMBIENT + DIFFUSE * up) + SPECULAR * pow(up, SHININESS);
    gl_FragColor = vec4(lit, color.a);
}"#;

pub fn snake_material() -> Result<Material> {
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
    .context("compiling snake shader")
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
