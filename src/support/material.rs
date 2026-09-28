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
use macroquad::math::vec4;
use macroquad::miniquad::{
    BlendFactor, BlendState, BlendValue, Equation, PipelineParams, ShaderSource, UniformDesc, UniformType,
};
use macroquad::texture::{FilterMode, Texture2D};

use crate::basic::Point;

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

// The light both shaders share: from straight above, with the eye there too,
// so all that counts is how much a surface faces up (`up`, 0..1). Diffuse
// light (Lambert) is that, and the specular highlight (Blinn-Phong) a power of
// it.
macro_rules! lighting {
    () => {
        r#"
const float AMBIENT = 0.45;
const float DIFFUSE = 0.55;
const float SPECULAR = 0.2;
const float SHININESS = 24.0;
vec3 light(vec3 base, float up) {
    return base * (AMBIENT + DIFFUSE * up) + SPECULAR * pow(up, SHININESS);
}
"#
    };
}

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
// round tube: uv.y runs across it, so x = 2·uv.y − 1 is where on the tube's
// cross-section the pixel is, and √(1 − x²) how much its surface faces up. The
// ends are hemispheres, like a pill's: along (0 where an end joins the body, 1
// at its tip) tilts the surface away along the body too, so it faces up by
// √(1 − along²)·√(1 − x²).
const FRAGMENT: &str = concat!(
    r#"#version 100
precision highp float;
varying vec2 uv;
varying vec2 seg_bounds;
varying float brightness;
varying float along;
uniform sampler2D Texture;
"#,
    lighting!(),
    r#"
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
    gl_FragColor = vec4(light(base, up), color.a);
}"#
);

pub fn snake_material() -> Result<Material> {
    load_material(
        ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
        MaterialParams {
            pipeline_params: alpha_blending(),
            ..Default::default()
        },
    )
    .context("compiling snake shader")
}

const BALL_VERTEX: &str = r#"#version 100
precision highp float;
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying vec2 disc;
varying vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    disc = texcoord;
    color = color0 / 255.0;
}"#;

// A ball in the vertex color, lit like the snake: disc is where on the ball
// the pixel is, from its center (0, 0) to its rim (length 1), so its surface
// faces up by √(1 − |disc|²).
const BALL_FRAGMENT: &str = concat!(
    r#"#version 100
precision highp float;
varying vec2 disc;
varying vec4 color;
"#,
    lighting!(),
    r#"
void main() {
    float up = sqrt(max(0.0, 1.0 - dot(disc, disc)));
    gl_FragColor = vec4(light(color.rgb, up), color.a);
}"#
);

/// Draws [`build_ball`] meshes (the apples) as balls lit like the snake.
///
/// [`build_ball`]: crate::support::mesh::build_ball
pub fn ball_material() -> Result<Material> {
    load_material(
        ShaderSource::Glsl {
            vertex: BALL_VERTEX,
            fragment: BALL_FRAGMENT,
        },
        MaterialParams {
            pipeline_params: alpha_blending(),
            ..Default::default()
        },
    )
    .context("compiling ball shader")
}

const LIT_VERTEX: &str = r#"#version 100
precision highp float;
attribute vec3 position;
attribute vec4 color0;
varying vec2 board;
varying vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    board = position.xy;
    color = color0 / 255.0;
}"#;

/// How many lights a [`light_material`] has room for.
pub const MAX_LIGHTS: usize = 10;

// Shapes in the dark, showing only where the lights reach. Each light is its
// position on the board and its intensity (0..1, in z); it falls off smoothly
// to nothing at `radius` from its center, and overlapping lights add up.
// MAX_LIGHTS is spliced in by light_material.
const LIT_FRAGMENT: &str = r#"#version 100
precision highp float;
varying vec2 board;
varying vec4 color;
uniform vec4 lights[MAX_LIGHTS];
uniform float radius;
void main() {
    float lit = 0.0;
    for (int k = 0; k < MAX_LIGHTS; k++) {
        vec2 d = (board - lights[k].xy) / radius;
        float f = max(0.0, 1.0 - dot(d, d));
        lit += lights[k].z * f * f;
    }
    gl_FragColor = vec4(color.rgb, color.a * min(lit, 1.0));
}"#;

/// Draws ordinary (vertex-colored) meshes in the dark, showing them only where
/// the lights set with [`set_lights`] reach: the light hints' grid and border.
pub fn light_material() -> Result<Material> {
    let fragment = LIT_FRAGMENT.replace("MAX_LIGHTS", &MAX_LIGHTS.to_string());
    load_material(
        ShaderSource::Glsl {
            vertex: LIT_VERTEX,
            fragment: &fragment,
        },
        MaterialParams {
            pipeline_params: alpha_blending(),
            uniforms: vec![
                UniformDesc::new("lights", UniformType::Float4).array(MAX_LIGHTS),
                UniformDesc::new("radius", UniformType::Float1),
            ],
            ..Default::default()
        },
    )
    .context("compiling light shader")
}

/// Set the lights (board position, intensity) of a [`light_material`], at
/// most [`MAX_LIGHTS`] of them, and how far each one reaches.
pub fn set_lights(material: &Material, lights: &[(Point, f32)], radius: f32) {
    assert!(
        lights.len() <= MAX_LIGHTS,
        "{} lights, room for {MAX_LIGHTS}",
        lights.len()
    );
    let mut uniform = [vec4(0., 0., 0., 0.); MAX_LIGHTS];
    for (slot, &(pos, intensity)) in uniform.iter_mut().zip(lights) {
        *slot = vec4(pos.x, pos.y, intensity, 0.);
    }
    material.set_uniform_array("lights", &uniform[..]);
    material.set_uniform("radius", radius);
}

/// The default alpha-over blending the standard mesh path uses, so
/// semi-transparent shapes composite the same way.
fn alpha_blending() -> PipelineParams {
    PipelineParams {
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    }
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
