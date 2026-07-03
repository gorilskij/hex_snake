//! Drawing primitives backed by macroquad + lyon tessellation.
//!
//! `MeshBuilder` accumulates polygons / lines / circles, tessellating each into
//! triangles (ggez did this internally; macroquad needs pre-triangulated
//! vertices). `Mesh::from_data` packs them into macroquad meshes, chunked to stay
//! under the u16 vertex-index limit.

use std::f32::consts::TAU;

use lyon_path::Path;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, LineCap, LineJoin, StrokeOptions,
    StrokeTessellator, StrokeVertex, VertexBuffers,
};
use macroquad::camera::{set_camera, set_default_camera, Camera2D};
use macroquad::color::Color as MqColor;
use macroquad::math::vec2;
use macroquad::models::{draw_mesh, Mesh as MqMesh, Vertex};
use macroquad::window::{clear_background, screen_height, screen_width};

use crate::basic::Point;
use crate::gfx::{Context, GameResult};

/// Drop-in replacement for `crate::gfx::graphics::Color` (named f32 fields so existing
/// struct literals and field access keep working).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[allow(dead_code)]
impl Color {
    pub const WHITE: Self = Self::new(1., 1., 1., 1.);
    pub const BLACK: Self = Self::new(0., 0., 0., 1.);
    pub const RED: Self = Self::new(1., 0., 0., 1.);
    pub const GREEN: Self = Self::new(0., 1., 0., 1.);
    pub const BLUE: Self = Self::new(0., 0., 1., 1.);
    pub const CYAN: Self = Self::new(0., 1., 1., 1.);
    pub const MAGENTA: Self = Self::new(1., 0., 1., 1.);
    pub const YELLOW: Self = Self::new(1., 1., 0., 1.);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r as f32 / 255., g as f32 / 255., b as f32 / 255., 1.)
    }

    pub const fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::new(r as f32 / 255., g as f32 / 255., b as f32 / 255., 1.)
    }
}

impl From<Color> for MqColor {
    fn from(c: Color) -> Self {
        MqColor::new(c.r, c.g, c.b, c.a)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum DrawMode {
    Fill,
    Stroke(f32),
}

impl DrawMode {
    pub fn fill() -> Self {
        DrawMode::Fill
    }

    pub fn stroke(width: f32) -> Self {
        DrawMode::Stroke(width)
    }
}

#[derive(Clone, Default)]
struct Primitive {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

#[derive(Default)]
pub struct MeshBuilder {
    primitives: Vec<Primitive>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn polygon(
        &mut self,
        mode: DrawMode,
        points: &[Point],
        color: impl Into<Color>,
    ) -> GameResult<&mut Self> {
        let color = color.into().into();
        let prim = match mode {
            DrawMode::Fill => tessellate_fill(points, color),
            DrawMode::Stroke(w) => tessellate_stroke(points, w, true, color),
        };
        self.primitives.push(prim);
        Ok(self)
    }

    pub fn circle(
        &mut self,
        mode: DrawMode,
        center: Point,
        radius: f32,
        _tolerance: f32,
        color: impl Into<Color>,
    ) -> GameResult<&mut Self> {
        let pts = circle_points(center, radius);
        self.polygon(mode, &pts, color)
    }

    pub fn line(&mut self, points: &[Point], width: f32, color: impl Into<Color>) -> GameResult<&mut Self> {
        let color = color.into().into();
        self.primitives.push(tessellate_stroke(points, width, false, color));
        Ok(self)
    }

    pub fn polyline(
        &mut self,
        mode: DrawMode,
        points: &[Point],
        color: impl Into<Color>,
    ) -> GameResult<&mut Self> {
        let width = match mode {
            DrawMode::Stroke(w) => w,
            DrawMode::Fill => 1.,
        };
        self.line(points, width, color)
    }

    pub fn build(&self) -> MeshData {
        MeshData { primitives: self.primitives.clone() }
    }
}

/// Tessellated geometry, not yet uploaded.
pub struct MeshData {
    primitives: Vec<Primitive>,
}

/// A drawable mesh, split into chunks that each stay under macroquad's
/// per-`draw_mesh` limits. macroquad's `quad_gl::geometry` **clamps** a single
/// draw call to `max_vertices` (default 10000) / `max_indices` (default 5000),
/// silently dropping any overflow — so a big mesh in one chunk loses its tail.
/// macroquad re-batches our smaller chunks across `draw_mesh` calls, so this is
/// cheap. Caps are kept comfortably below the defaults.
pub struct Mesh {
    meshes: Vec<MqMesh>,
}

const MAX_VERTS: usize = 9000;
const MAX_INDICES: usize = 4500;

impl Mesh {
    pub fn from_data(_ctx: &Context, data: MeshData) -> Mesh {
        let mut meshes = vec![];
        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u16> = vec![];

        for prim in data.primitives {
            if prim.vertices.is_empty() {
                continue;
            }
            let over_verts = vertices.len() + prim.vertices.len() > MAX_VERTS;
            let over_indices = indices.len() + prim.indices.len() > MAX_INDICES;
            if (over_verts || over_indices) && !vertices.is_empty() {
                meshes.push(MqMesh {
                    vertices: std::mem::take(&mut vertices),
                    indices: std::mem::take(&mut indices),
                    texture: None,
                });
            }
            let base = vertices.len() as u16;
            vertices.extend(prim.vertices);
            indices.extend(prim.indices.into_iter().map(|i| i + base));
        }

        if !vertices.is_empty() {
            meshes.push(MqMesh { vertices, indices, texture: None });
        }

        Mesh { meshes }
    }
}

#[derive(Copy, Clone)]
pub struct DrawParam {
    pub dest: Point,
    pub color: Color,
}

#[allow(clippy::should_implement_trait)]
impl DrawParam {
    pub fn default() -> Self {
        Self {
            dest: Point::zero(),
            color: Color::WHITE,
        }
    }

    pub fn dest(mut self, dest: impl Into<Point>) -> Self {
        self.dest = dest.into();
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }
}

/// Mirrors `ggez::graphics::Canvas`. Drawing happens immediately under macroquad;
/// this just clears the frame and sets the board-offset camera per draw.
pub struct Canvas;

impl Canvas {
    pub fn from_frame(_ctx: &mut Context, clear: impl Into<Color>) -> Canvas {
        clear_background(clear.into().into());
        Canvas
    }

    pub fn draw(&mut self, mesh: &Mesh, param: DrawParam) {
        set_board_camera(param.dest);
        for m in &mesh.meshes {
            draw_mesh(m);
        }
    }

    pub fn finish(&mut self, _ctx: &mut Context) -> GameResult {
        set_default_camera();
        Ok(())
    }
}

/// Set a pixel-coordinate camera (top-left origin, y down) translated by `dest`,
/// so board-local vertices land at `vertex + dest` on screen.
pub fn set_board_camera(dest: Point) {
    let w = screen_width();
    let h = screen_height();
    let cam = Camera2D {
        target: vec2(w / 2. - dest.x, h / 2. - dest.y),
        // macroquad's NDC->screen for draw_mesh is y-down, so a positive zoom.y
        // keeps world-y increasing downward (board top at y=0). A negative zoom.y
        // flips the board vertically (snake moving up renders as moving down).
        zoom: vec2(2. / w, 2. / h),
        offset: vec2(0., 0.),
        rotation: 0.,
        render_target: None,
        viewport: None,
    };
    set_camera(&cam);
}

fn to_primitive(buffers: VertexBuffers<[f32; 2], u16>, color: MqColor) -> Primitive {
    let vertices = buffers
        .vertices
        .iter()
        .map(|[x, y]| Vertex::new(*x, *y, 0., 0., 0., color))
        .collect();
    Primitive { vertices, indices: buffers.indices }
}

fn tessellate_fill(points: &[Point], color: MqColor) -> Primitive {
    if points.len() < 3 {
        return Primitive::default();
    }
    let mut pb = Path::builder();
    pb.begin(lyon_path::math::point(points[0].x, points[0].y));
    for p in &points[1..] {
        pb.line_to(lyon_path::math::point(p.x, p.y));
    }
    pb.end(true);
    let path = pb.build();

    let mut buffers: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
    let mut tess = FillTessellator::new();
    let _ = tess.tessellate_path(
        &path,
        &FillOptions::default(),
        &mut BuffersBuilder::new(&mut buffers, |v: FillVertex| {
            let p = v.position();
            [p.x, p.y]
        }),
    );
    to_primitive(buffers, color)
}

fn tessellate_stroke(points: &[Point], width: f32, closed: bool, color: MqColor) -> Primitive {
    if points.len() < 2 {
        return Primitive::default();
    }
    let mut pb = Path::builder();
    pb.begin(lyon_path::math::point(points[0].x, points[0].y));
    for p in &points[1..] {
        pb.line_to(lyon_path::math::point(p.x, p.y));
    }
    pb.end(closed);
    let path = pb.build();

    let opts = StrokeOptions::default()
        .with_line_width(width)
        .with_line_cap(LineCap::Round)
        .with_line_join(LineJoin::Round);

    let mut buffers: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
    let mut tess = StrokeTessellator::new();
    let _ = tess.tessellate_path(
        &path,
        &opts,
        &mut BuffersBuilder::new(&mut buffers, |v: StrokeVertex| {
            let p = v.position();
            [p.x, p.y]
        }),
    );
    to_primitive(buffers, color)
}

fn circle_points(center: Point, radius: f32) -> Vec<Point> {
    let n = (radius.max(4.) as usize).clamp(12, 64);
    (0..n)
        .map(|i| {
            let a = i as f32 / n as f32 * TAU;
            Point {
                x: center.x + radius * a.cos(),
                y: center.y + radius * a.sin(),
            }
        })
        .collect()
}
