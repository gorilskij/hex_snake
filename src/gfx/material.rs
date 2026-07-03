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

use macroquad::color::Color as MqColor;
use macroquad::material::{gl_use_default_material, gl_use_material, load_material, Material, MaterialParams};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, PipelineParams, ShaderSource};
use macroquad::texture::{FilterMode, Texture2D};

use crate::gfx::graphics::Color;
use crate::gfx::{GameError, GameResult};

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}"#;

// Samples the palette LUT by uv.x ("distance along body"). The LUT is a
// single-row texture, so v is fixed at 0.5. uv.y (across-width) is currently
// ignored, matching the old flat-across-width look; it is available for future
// shading.
const FRAGMENT: &str = r#"#version 100
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    gl_FragColor = texture2D(Texture, vec2(uv.x, 0.5));
}"#;

/// The shader that colors snakes from a palette LUT. One instance is shared by
/// every snake; each snake binds its own LUT (as the mesh's texture) per frame.
pub struct SnakeMaterial {
    material: Material,
}

impl SnakeMaterial {
    pub fn new() -> GameResult<Self> {
        let material = load_material(
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
        .map_err(|e| GameError(format!("failed to load snake material: {e:?}")))?;
        Ok(Self { material })
    }

    /// Route subsequent `draw_mesh` calls through this shader.
    pub fn bind(&self) {
        gl_use_material(&self.material);
    }

    /// Restore macroquad's default material.
    pub fn unbind(&self) {
        gl_use_default_material();
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

// ---------------------------------------------------------------------------
// Stage 0 spike — temporary validation of the shader + LUT path on native and
// wasm. Remove once snake integration (Stage 3) is verified.
// ---------------------------------------------------------------------------

/// A rainbow LUT for the spike test.
#[allow(dead_code)]
pub fn spike_rainbow_lut(n: usize) -> PaletteLut {
    let colors: Vec<Color> = (0..n)
        .map(|i| {
            let h = i as f32 / n as f32 * 6.0;
            let x = 1.0 - ((h % 2.0) - 1.0).abs();
            let (r, g, b) = match h as u32 {
                0 => (1., x, 0.),
                1 => (x, 1., 0.),
                2 => (0., 1., x),
                3 => (0., x, 1.),
                4 => (x, 0., 1.),
                _ => (1., 0., x),
            };
            Color::new(r, g, b, 1.)
        })
        .collect();
    PaletteLut::new(&colors)
}

/// Draw a screen-space quad whose horizontal axis maps `uv.x` 0→1, colored via
/// the LUT + shader. If this shows a smooth rainbow, the whole pipeline works.
#[allow(dead_code)]
pub fn spike_draw_test_quad(mat: &SnakeMaterial, lut: &PaletteLut) {
    use macroquad::camera::set_default_camera;
    use macroquad::models::{draw_mesh, Mesh as MqMesh, Vertex};

    set_default_camera();
    let w = MqColor::new(1., 1., 1., 1.);
    let mesh = MqMesh {
        vertices: vec![
            Vertex::new(100., 100., 0., 0., 0., w),
            Vertex::new(600., 100., 0., 1., 0., w),
            Vertex::new(600., 400., 0., 1., 1., w),
            Vertex::new(100., 400., 0., 0., 1., w),
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: Some(lut.texture()),
    };
    mat.bind();
    draw_mesh(&mesh);
    mat.unbind();
}
