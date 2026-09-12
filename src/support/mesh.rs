//! Mesh building + drawing primitives backed by macroquad + lyon tessellation.
//!
//! Each `build_*` free function tessellates one shape (polygon / line / circle /
//! shaded ribbon) into triangles and returns a self-contained [`Mesh`] (macroquad
//! needs pre-triangulated vertices). [`Mesh::combine`] packs several such meshes
//! into one, chunked to stay under macroquad's per-`draw_mesh` clamp.

use std::f32::consts::TAU;
use std::mem::take;

use lyon_path::Path;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, LineCap, LineJoin, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};
use macroquad::camera::{set_camera, Camera2D};
use macroquad::color::Color as MqColor;
use macroquad::material::{gl_use_default_material, gl_use_material, Material};
use macroquad::math::{vec2, vec4};
use macroquad::models::{draw_mesh, Mesh as MqMesh, Vertex};
use macroquad::texture::Texture2D;
use macroquad::window::{screen_height, screen_width};

use crate::basic::Point;

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

/// Build a flat-colored polygon: `Fill` triangulates the interior, `Stroke(w)`
/// outlines it with a closed stroke of width `w`.
pub fn build_polygon(mode: DrawMode, points: &[Point], color: impl Into<MqColor>) -> Mesh {
    let color = color.into();
    let (vertices, indices) = match mode {
        DrawMode::Fill => tessellate_fill(points, color),
        DrawMode::Stroke(w) => tessellate_stroke(points, w, true, color),
    };
    Mesh::raw(vertices, indices)
}

/// Build a flat-colored circle, approximated as a regular polygon.
pub fn build_circle(mode: DrawMode, center: Point, radius: f32, color: impl Into<MqColor>) -> Mesh {
    build_polygon(mode, &circle_points(center, radius), color)
}

/// Build a flat-colored open stroke (polyline) of width `width`.
pub fn build_line(points: &[Point], width: f32, color: impl Into<MqColor>) -> Mesh {
    let (vertices, indices) = tessellate_stroke(points, width, false, color.into());
    Mesh::raw(vertices, indices)
}

/// Build an open polyline; `Stroke(w)` sets the width, `Fill` falls back to 1px.
pub fn build_polyline(mode: DrawMode, points: &[Point], color: impl Into<MqColor>) -> Mesh {
    let width = match mode {
        DrawMode::Stroke(w) => w,
        DrawMode::Fill => 1.,
    };
    build_line(points, width, color)
}

/// Build a convex polygon with a color per vertex, interpolated across it.
pub fn build_colored_polygon(points: &[(Point, MqColor)]) -> Mesh {
    if points.len() < 3 {
        return Mesh::empty();
    }
    let vertices = points
        .iter()
        .map(|&(p, color)| Vertex::new(p.x, p.y, 0., 0., 0., color))
        .collect();
    // a fan around the first vertex
    let indices = (1..points.len() as u16 - 1).flat_map(|i| [0, i, i + 1]).collect();
    Mesh::raw(vertices, indices)
}

/// Build a filled polygon whose color comes from a shader (via a palette LUT)
/// rather than a flat vertex color. `default_points` are the polygon outline in
/// the segment's *default orientation*; `uv_of` maps each tessellated vertex
/// (still in default orientation) to its `(u, v)` texture coords, and
/// `transform` places the vertex on the board. Computing uv before the transform
/// keeps it in body space (rotation/translation invariant); the vertex color is
/// unused by the snake shader, so it is left white. `brightness` scales the sampled color.
pub fn build_shaded_polygon<U, T>(default_points: &[Point], brightness: f32, uv_of: U, transform: T) -> Mesh
where
    U: Fn(Point) -> (f32, f32),
    T: Fn(Point) -> Point,
{
    let (positions, indices) = tessellate_fill_positions(default_points);
    if positions.is_empty() {
        return Mesh::empty();
    }
    let white = MqColor::new(1., 1., 1., 1.);
    // seg_bounds (0,1) → the shader's clamp is a no-op (flat color per poly)
    let bounds = vec4(0., 1., brightness, 0.);
    let vertices = positions
        .into_iter()
        .map(|p| {
            let (u, v) = uv_of(p);
            let tp = transform(p);
            let mut vert = Vertex::new(tp.x, tp.y, 0., u, v, white);
            vert.normal = bounds;
            vert
        })
        .collect();
    Mesh::raw(vertices, indices)
}

/// Build a shaded triangle-strip ribbon. Each entry is a cross-section
/// `(inner, outer, frac)` in default orientation: `inner`/`outer` are the two
/// edge points and `frac` is the segment-local body fraction. `u_of` maps `frac`
/// to the global `uv.x`; `transform` places points on the board. Adjacent
/// cross-sections share vertices, so radial edges are single constant-`frac`
/// lines (seamless) — even when `inner` collapses to one pivot point, which is
/// then reused across cross-sections with distinct uv.
///
/// `seg_bounds` is this segment's `(lo, hi)` uv range; it is passed to every
/// vertex (in `normal.xy`) so the shader can clamp the LUT lookup to it, keeping
/// color boundaries on the geometry seam instead of a texel. `brightness`
/// (passed in `normal.z`) scales the sampled color.
pub fn build_shaded_ribbon<U, T>(
    cross_sections: &[(Point, Point, f32)],
    seg_bounds: (f32, f32),
    brightness: f32,
    u_of: U,
    transform: T,
) -> Mesh
where
    U: Fn(f32) -> f32,
    T: Fn(Point) -> Point,
{
    if cross_sections.len() < 2 {
        return Mesh::empty();
    }
    let white = MqColor::new(1., 1., 1., 1.);
    let bounds = vec4(seg_bounds.0, seg_bounds.1, brightness, 0.);
    let mut vertices = Vec::with_capacity(cross_sections.len() * 2);
    for &(inner, outer, frac) in cross_sections {
        let u = u_of(frac);
        let ti = transform(inner);
        let to = transform(outer);
        let mut vi = Vertex::new(ti.x, ti.y, 0., u, 0., white);
        let mut vo = Vertex::new(to.x, to.y, 0., u, 1., white);
        vi.normal = bounds;
        vo.normal = bounds;
        vertices.push(vi);
        vertices.push(vo);
    }
    let mut indices = Vec::with_capacity((cross_sections.len() - 1) * 6);
    for k in 0..cross_sections.len() - 1 {
        let i = (k * 2) as u16;
        // inner_k = i, outer_k = i+1, inner_{k+1} = i+2, outer_{k+1} = i+3
        indices.extend_from_slice(&[i, i + 1, i + 3, i, i + 3, i + 2]);
    }
    Mesh::raw(vertices, indices)
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
    /// A mesh that draws nothing.
    pub fn empty() -> Mesh {
        Mesh { meshes: vec![] }
    }

    /// Wrap one tessellated shape (a single chunk). A single shape never
    /// approaches the per-draw clamp, so no splitting is needed here — that
    /// happens in [`Mesh::combine`] when many shapes are packed together.
    fn raw(vertices: Vec<Vertex>, indices: Vec<u16>) -> Mesh {
        if vertices.is_empty() {
            Mesh::empty()
        } else {
            Mesh {
                meshes: vec![MqMesh { vertices, indices, texture: None }],
            }
        }
    }

    /// Pack several per-shape meshes into one, re-batching their chunks so each
    /// stays under the per-`draw_mesh` clamp (rather than one draw call per
    /// shape). Textures are dropped — apply one afterwards with [`set_texture`]
    /// if needed (all callers build untextured shapes and texture the whole).
    ///
    /// [`set_texture`]: Mesh::set_texture
    pub fn combine(parts: impl IntoIterator<Item = Mesh>) -> Mesh {
        let mut meshes = vec![];
        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u16> = vec![];

        for chunk in parts.into_iter().flat_map(|m| m.meshes) {
            if chunk.vertices.is_empty() {
                continue;
            }
            let over_verts = vertices.len() + chunk.vertices.len() > MAX_VERTS;
            let over_indices = indices.len() + chunk.indices.len() > MAX_INDICES;
            if (over_verts || over_indices) && !vertices.is_empty() {
                meshes.push(MqMesh {
                    vertices: take(&mut vertices),
                    indices: take(&mut indices),
                    texture: None,
                });
            }
            let base = vertices.len() as u16;
            vertices.extend(chunk.vertices);
            indices.extend(chunk.indices.into_iter().map(|i| i + base));
        }

        if !vertices.is_empty() {
            meshes.push(MqMesh { vertices, indices, texture: None });
        }

        Mesh { meshes }
    }

    /// Attach a texture (e.g. a snake's palette LUT) to every chunk, so the
    /// shader's `Texture` sampler resolves to it when the mesh is drawn.
    pub fn set_texture(&mut self, texture: Texture2D) {
        for m in &mut self.meshes {
            m.texture = Some(texture.clone());
        }
    }

    /// Draws the mesh's chunks on the default material. The caller must have
    /// set the **board camera** ([`set_board_camera`]) beforehand — the
    /// vertices are in board-local space.
    pub fn draw(&self) {
        for m in &self.meshes {
            draw_mesh(m);
        }
    }

    /// Draws the mesh's chunks through `material` (e.g. the snake shader). As
    /// with [`Mesh::draw`], the caller must have set the **board camera**
    /// beforehand.
    pub fn draw_shaded(&self, material: &Material) {
        gl_use_material(material);
        for m in &self.meshes {
            draw_mesh(m);
        }
        gl_use_default_material();
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

/// Triangulate a filled polygon, returning the vertex positions and indices
/// (no color/uv). Shared by the flat-color and shaded paths.
fn tessellate_fill_positions(points: &[Point]) -> (Vec<Point>, Vec<u16>) {
    if points.len() < 3 {
        return (vec![], vec![]);
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
    let positions = buffers.vertices.iter().map(|[x, y]| Point { x: *x, y: *y }).collect();
    (positions, buffers.indices)
}

fn tessellate_fill(points: &[Point], color: MqColor) -> (Vec<Vertex>, Vec<u16>) {
    let (positions, indices) = tessellate_fill_positions(points);
    let vertices = positions
        .into_iter()
        .map(|p| Vertex::new(p.x, p.y, 0., 0., 0., color))
        .collect();
    (vertices, indices)
}

fn tessellate_stroke(points: &[Point], width: f32, closed: bool, color: MqColor) -> (Vec<Vertex>, Vec<u16>) {
    if points.len() < 2 {
        return (vec![], vec![]);
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
    let vertices = buffers
        .vertices
        .iter()
        .map(|[x, y]| Vertex::new(*x, *y, 0., 0., 0., color))
        .collect();
    (vertices, buffers.indices)
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
