//! Hints at the board's edges about wrapping around the board, in one of three
//! styles, easing in, out and between values in real time.
//!
//! [`HintStyle::Border`] and [`HintStyle::Gradient`] both say *what lies on the
//! other side*: every side of a border cell that wraps shows what would happen
//! to the player on crossing it (see [`Outcome`]), either by recoloring that
//! stretch of the border or as a gradient fading from it into the cell.
//!
//! [`HintStyle::Lines`] and [`HintStyle::SmoothLines`] say *which way out*: one
//! line along each of the three axes, from under the head out to the border it
//! would leave through either way, drawn over the snake so a line is never
//! hidden by the body it crosses. The first snaps to the head's cell, the
//! second follows the head itself.
//!
//! [`HintStyle::Teleport`] instead says *where the head would come out*. For
//! each direction it could keep going in, it recolors both ends of that wrap —
//! the border it would leave through and the one it would arrive at. Nearer in,
//! the arrival end also grows a triangle pointing into its cell, so a wrap the
//! head is about to take reads quite differently from one merely in range.

use std::collections::HashMap;

use macroquad::color::Color;

use crate::app::game_context::GameContext;
use crate::app::palette::{HintColors, Palette, TeleportHintColors};
use crate::app::prefs::HintStyle;
use crate::app::screen::Environment;
use crate::app::snake_management::{Outcome, outcome_at};
use crate::basic::{CellDim, Dir, HexDim, HexPoint, Point, board};
use crate::color::lerp;
use crate::rendering::segments::centerline::Centerline;
use crate::rendering::shape::{Hexagon, Shape};
use crate::snake::Body;
use crate::support::mesh::{DrawMode, Mesh, build_colored_polygon, build_line, build_polygon};
use crate::support::time::Instant;

/// Roughly how long (s) a hint takes to settle after what's behind it changes.
const FADE_TIME: f32 = 0.1;

/// How far a gradient hint reaches into its cell, as a fraction of the way from
/// the edge to the cell's center.
const GRADIENT_DEPTH: f32 = 0.5;

/// How far from a wrap the head can be before either end of it stops being
/// marked, in cells. Marks are solid throughout; only the triangle's depth
/// tracks the distance.
const TELEPORT_RANGE: f32 = 10.;

/// How far from a wrap its exit triangle starts growing, in cells. Between here
/// and [`TELEPORT_RANGE`] both ends are marked but flat.
const TELEPORT_TRIANGLE_RANGE: f32 = 5.;

/// How far a teleport triangle reaches at full strength, as a fraction of the
/// way from the border's inner edge to the cell's center.
const TELEPORT_DEPTH: f32 = 0.5;

/// What an edge has to say right now.
#[derive(Copy, Clone, Debug)]
struct Mark {
    color: Color,
    /// How far a teleport triangle reaches into the cell, from 0 at the border's
    /// inner edge to 1 at [`TELEPORT_DEPTH`] of the way to the center. Always 0
    /// for the outcome styles, which draw no triangle.
    depth: f32,
}

/// An edge's mark plus how far it has eased in.
#[derive(Copy, Clone, Debug)]
struct Hint {
    mark: Mark,
    /// How far eased in, 0 to 1. This is the only thing that moves in real
    /// time, and only when an edge gains or loses its mark.
    fade: f32,
}

impl Hint {
    fn new(mark: Mark) -> Self {
        Self { mark, fade: 0. }
    }

    /// Take on `target`, or start losing the mark if there is none.
    ///
    /// The mark's depth is taken as given rather than eased, so a teleport
    /// triangle follows the head exactly instead of trailing a tick behind it.
    /// Color is the exception: an outcome can change under an edge that keeps
    /// its hint, and that should slide rather than jump.
    fn approach(&mut self, target: Option<Mark>, step: f32) {
        match target {
            Some(target) => {
                self.mark = Mark {
                    color: lerp(self.mark.color, target.color, step),
                    depth: target.depth,
                };
                self.fade += (1. - self.fade) * step;
            }
            None => self.fade -= self.fade * step,
        }
    }

    /// The mark as it should be drawn, eased out towards `absent`.
    fn drawn(&self, absent: Mark) -> Mark {
        Mark {
            color: lerp(absent.color, self.mark.color, self.fade),
            depth: absent.depth + (self.mark.depth - absent.depth) * self.fade,
        }
    }
}

pub struct BorderHints {
    /// Every edge whose hint is (still) visible.
    hints: HashMap<(HexPoint, Dir), Hint>,
    last_update: Option<Instant>,
}

impl BorderHints {
    pub fn new() -> Self {
        Self {
            hints: HashMap::new(),
            last_update: None,
        }
    }

    /// Drop all hints at once (e.g. when the board changes shape).
    pub fn clear(&mut self) {
        self.hints.clear();
    }

    /// Ease every hint towards what its edge currently has to say, and build the
    /// mesh in the given style.
    pub fn mesh(&mut self, env: &Environment, player_idx: usize, style: HintStyle) -> Mesh {
        let palette = &env.gtx.palette;
        if style == HintStyle::None {
            self.clear();
            return Mesh::empty();
        }

        let now = Instant::now();
        let elapsed = self.last_update.map_or(0., |last| (now - last).as_secs_f32());
        self.last_update = Some(now);
        // exponential approach, ~95% of the way there after FADE_TIME
        let step = 1. - (-3. * elapsed / FADE_TIME).exp();

        if let HintStyle::Lines | HintStyle::SmoothLines = style {
            self.clear();
            let body = &env.snakes[player_idx].body;
            return line_hints(
                body,
                &env.gtx,
                style == HintStyle::SmoothLines,
                palette.hint_line_color,
                palette.hint_line_thickness,
            );
        }

        let targets = match style {
            HintStyle::Border => outcome_hints(env, player_idx, palette.border_hint_colors),
            HintStyle::Gradient => outcome_hints(env, player_idx, palette.gradient_hint_colors),
            HintStyle::Teleport => {
                let body = &env.snakes[player_idx].body;
                teleport_hints(body, env.gtx.board_dim, palette.teleport_hint_colors)
            }
            HintStyle::Lines | HintStyle::SmoothLines | HintStyle::None => unreachable!("returned above"),
        };

        let draw_border = env.gtx.prefs.draw_border;
        let absent = |mark: Mark| absent_mark(style, palette, draw_border, mark);

        for (edge, target) in &targets {
            self.hints.entry(*edge).or_insert(Hint::new(*target));
        }
        self.hints.retain(|edge, hint| {
            let target = targets.get(edge).copied();
            hint.approach(target, step);
            target.is_some() || hint.fade > 1e-3
        });

        let cell_dim = env.gtx.cell_dim;
        let board_dim = env.gtx.board_dim;
        let half_width = palette.border_thickness / 2.;
        // corners clockwise from the top-left: side i (corner i to i + 1) faces Dir i
        let corners = Hexagon::raw_points(cell_dim);
        let center = Hexagon::center(cell_dim);

        Mesh::combine(self.hints.iter().map(|(&(pos, dir), hint)| {
            let mark = hint.drawn(absent(hint.mark));
            let origin = pos.to_cartesian(cell_dim);
            let corner = |i: usize| origin + corners[i % 6];
            let side = dir as usize;
            let (start, end) = (corner(side), corner(side + 1));

            match style {
                HintStyle::Border | HintStyle::Teleport => {
                    // A band the border's width, centered on the side like the
                    // border itself, with each end cut along the line halving the
                    // corner the border turns at, so it meets the next stretch of
                    // border (hinted or not) exactly. If the cell's other side at
                    // that corner is on the border too, the border turns around
                    // this cell and the cut points at its center; otherwise it
                    // turns onto the neighboring cell and the cut runs along the
                    // side between the two.
                    let on_border = |side: usize| !board_dim.contains(pos.translate(Dir::from(side as u8), 1));
                    let cut = |vertex: Point, other_side: usize, other_corner: Point| {
                        if on_border(other_side) {
                            origin + center - vertex
                        } else {
                            other_corner - vertex
                        }
                    };

                    let along = (end - start) / (end - start).magnitude();
                    let normal = Point { x: -along.y, y: along.x };
                    // the points on a cut (through `vertex`) half a width to either side
                    let cut_ends = |vertex: Point, cut: Point| {
                        let reach = cut * (half_width / (cut.x * normal.x + cut.y * normal.y));
                        (vertex + reach, vertex - reach)
                    };

                    let (start_left, start_right) = cut_ends(start, cut(start, side + 5, corner(side + 5)));
                    let (end_left, end_right) = cut_ends(end, cut(end, side + 1, corner(side + 2)));
                    let band = build_polygon(
                        DrawMode::Fill,
                        &[start_left, end_left, end_right, start_right],
                        mark.color,
                    );

                    if style == HintStyle::Border {
                        return band;
                    }

                    // The triangle sits just inside the border rather than
                    // overlapping it, so its base is the band's inner edge —
                    // whichever of the two cut pairs faces the cell's center.
                    let inward = origin + center;
                    let inner = |near: Point, far: Point| {
                        if (near - inward).magnitude() <= (far - inward).magnitude() {
                            near
                        } else {
                            far
                        }
                    };
                    let base_start = inner(start_left, start_right);
                    let base_end = inner(end_left, end_right);
                    let base_mid = (base_start + base_end) * 0.5;
                    let apex = base_mid + (inward - base_mid) * (TELEPORT_DEPTH * mark.depth);

                    Mesh::combine([
                        band,
                        build_polygon(DrawMode::Fill, &[base_start, base_end, apex], mark.color),
                    ])
                }
                HintStyle::Gradient => {
                    let inward = |corner: Point| corner + (origin + center - corner) * GRADIENT_DEPTH;
                    let clear = Color { a: 0., ..mark.color };
                    build_colored_polygon(&[
                        (start, mark.color),
                        (end, mark.color),
                        (inward(end), clear),
                        (inward(start), clear),
                    ])
                }
                HintStyle::Lines | HintStyle::SmoothLines | HintStyle::None => unreachable!("returned above"),
            }
        }))
    }
}

/// What an edge with nothing to say looks like, which is what a hint eases out
/// towards as it loses its mark.
///
/// The outcome styles sit beside the border and fade their own color out. A
/// mark painted *over* the border stays opaque instead, and eases back into
/// whatever it is painted over: the border's own color while the border is
/// drawn, the board's background once it is off — otherwise turning the border
/// off would leave the mark fading into a line that is not there.
fn absent_mark(style: HintStyle, palette: &Palette, draw_border: bool, mark: Mark) -> Mark {
    match style {
        HintStyle::Teleport => Mark {
            color: if draw_border {
                palette.border_color
            } else {
                palette.background_color
            },
            depth: 0.,
        },
        _ => Mark {
            color: Color { a: 0., ..mark.color },
            depth: 0.,
        },
    }
}

/// A line along each of the three axes, from the head out to the borders.
///
/// Each axis is one polyline through the head, so the two ends are the two
/// borders it would leave through going either way along it.
///
/// [`HintStyle::Lines`] snaps the origin to the head's cell center, so the
/// lines step from cell to cell; [`HintStyle::SmoothLines`] puts it on the tip
/// of the drawn head, and the lines slide with it. An origin off the cell center
/// is what makes the ends interesting: they are no longer the middle of a
/// border cell's outer side, so each one is found by clipping the ray against
/// the board's own edge (see [`ray_to_border`]) and slides along it as the head
/// moves.
fn line_hints(body: &Body, gtx: &GameContext, smooth: bool, color: Color, thickness: f32) -> Mesh {
    let (board_dim, cell_dim) = (gtx.board_dim, gtx.cell_dim);
    let head = body.segments[0].pos;
    let mut origin = head.to_cartesian(cell_dim) + Hexagon::center(cell_dim);
    if smooth {
        // The tip of the drawn head, taken from the renderer's own centerline
        // rather than guessed from the head's fraction along its cell: through
        // a turn the head travels an arc, so anything derived from the cell's
        // axis cuts the corner and the lines snap as the turn ends.
        if let Some(tip) = Centerline::of(body, gtx).head_tip() {
            origin = tip;
        }
    }

    let lines = line_hint_points(origin, head, board_dim, cell_dim);
    Mesh::combine(lines.map(|points| build_line(&points, thickness, color)))
}

/// The three lines, each as the polyline `[one border, the head, the other]`.
///
/// One per axis, so only half the directions are walked; the other half is the
/// same line's other end.
fn line_hint_points(origin: Point, head: HexPoint, board_dim: HexDim, cell_dim: CellDim) -> [[Point; 3]; 3] {
    let border = border_segments(board_dim, cell_dim);
    [Dir::U, Dir::Ul, Dir::Ur].map(|dir| {
        [
            ray_to_border(origin, dir, head, &border, board_dim, cell_dim),
            origin,
            ray_to_border(origin, -dir, head, &border, board_dim, cell_dim),
        ]
    })
}

/// Every side of the board's edge, as a segment and the way out through it.
/// These are exactly the sides that wrap: a side is on the edge precisely when
/// what lies across it is on the other side of the board.
fn border_segments(board_dim: HexDim, cell_dim: CellDim) -> Vec<(Point, Point, Dir)> {
    let corners = Hexagon::raw_points(cell_dim);
    wrap_edges(board_dim)
        .map(|(pos, dir, _)| {
            // side i runs from corner i to corner i + 1 and faces Dir i
            let origin = pos.to_cartesian(cell_dim);
            let side = dir as usize;
            (origin + corners[side], origin + corners[(side + 1) % 6], dir)
        })
        .collect()
}

/// Where a ray from `origin` along `dir` leaves the board.
///
/// The board's edge zigzags, so the ray is clipped against it segment by
/// segment and the nearest hit wins, rather than assuming which cell's side it
/// comes out of: from anywhere but a cell center it can leave through a
/// neighbouring cell's side instead.
///
/// Only sides the ray is *leaving* through count. A head on the border itself
/// is on two of them at once, and the one it is heading into ends the line
/// right there (a line of no length), while the one behind it must not end the
/// line going the other way.
///
/// The fallback — a ray that grazes the edge between two segments and misses
/// both — steps out along `head`'s lane of cells instead, as if from its center.
fn ray_to_border(
    origin: Point,
    dir: Dir,
    head: HexPoint,
    border: &[(Point, Point, Dir)],
    board_dim: HexDim,
    cell_dim: CellDim,
) -> Point {
    let step = board::cartesian_step(dir, cell_dim);

    // origin + t * step = a + u * (b - a), for the smallest t ahead of the head
    let hit = border
        .iter()
        .filter(|&&(.., out)| {
            // pointing out through this side, not in through it
            let out = board::cartesian_step(out, cell_dim);
            step.x * out.x + step.y * out.y > 0.
        })
        .filter_map(|&(a, b, _)| {
            let along = b - a;
            let denominator = step.x * along.y - step.y * along.x;
            if denominator.abs() < 1e-6 {
                // parallel to this side, so either no crossing or a graze
                return None;
            }
            let offset = a - origin;
            let t = (offset.x * along.y - offset.y * along.x) / denominator;
            let u = (offset.x * step.y - offset.y * step.x) / denominator;
            // the ends are shared with the next side, so a corner counts
            (t >= 0. && (-1e-6..=1. + 1e-6).contains(&u)).then_some(t)
        })
        .fold(f32::INFINITY, f32::min);

    if hit.is_finite() {
        return origin + step * hit;
    }

    let mut cells = 0.;
    let mut cell = head;
    while board_dim.contains(cell.translate(dir, 1)) {
        cell = cell.translate(dir, 1);
        cells += 1.;
    }
    origin + step * (cells + 0.5)
}

/// What the player would run into across each wrapping edge, marked on the edge
/// itself.
fn outcome_hints(env: &Environment, player_idx: usize, colors: HintColors) -> HashMap<(HexPoint, Dir), Mark> {
    wrap_edges(env.gtx.board_dim)
        .filter_map(|(pos, dir, destination)| {
            let color = match outcome_at(env, player_idx, destination)? {
                Outcome::Apple => colors.apple,
                Outcome::BadApple => colors.bad_apple,
                Outcome::Pass => colors.pass,
                Outcome::Cut => colors.cut,
                Outcome::Crash => colors.crash,
            };
            Some(((pos, dir), Mark { color, depth: 0. }))
        })
        .collect()
}

/// Both ends of every wrap the head could reach: the border it would leave
/// through, and the one it would come out of.
///
/// One wrap per direction: going straight in `dir` the head runs off the board
/// after some number of steps, from the cell this calls the *entry*. Crossing
/// lands it in `destination` still going `dir`, so it emerges through that
/// cell's `-dir` side — the *exit*, on the border too, being the other half of
/// the same wrap.
///
/// Both ends of a wrap are colored by how close the head is, on the same
/// quantity that opens the triangle: purple while the wrap is merely in range,
/// sweeping down through the hues to red as the head arrives.
///
/// How many marks that comes to is left to the range: usually three wraps,
/// since the two along an axis are a board apart and only one of each pair is
/// close, but four in a corner, where a diagonal through the board is a single
/// cell and so leaves it in both directions at once.
fn teleport_hints(body: &Body, board_dim: HexDim, colors: TeleportHintColors) -> HashMap<(HexPoint, Dir), Mark> {
    let head = body.segments[0].pos;

    let wraps: Vec<(HexPoint, Dir, HexPoint, f32)> = Dir::iter()
        .filter_map(|dir| {
            // whole cells from the head's cell to the one it would cross from
            let mut entry = head;
            let mut cells = 0;
            while board_dim.contains(entry.translate(dir, 1)) {
                entry = entry.translate(dir, 1);
                cells += 1;
            }

            // Plus however much of its own cell the head has left to cross, so
            // the distance tracks the head continuously rather than jumping a
            // whole cell at a time. Progress made heading one way only counts
            // towards `dir` by the cosine between them; going the other way it
            // counts against, and the wrap behind recedes.
            let progress = body.head_fraction * body.dir.clockwise_angle_to(dir).cos();
            let distance = cells as f32 + 1. - progress;
            if distance > TELEPORT_RANGE {
                return None;
            }

            // The triangle only opens over the last stretch, and quadratically,
            // so it creeps at first and rushes as the head arrives.
            let closeness = ((TELEPORT_TRIANGLE_RANGE - distance) / TELEPORT_TRIANGLE_RANGE).max(0.);
            let destination = entry.wrap_destination(dir, board_dim)?;
            Some((entry, dir, destination, closeness * closeness))
        })
        .collect();

    // Exits go in second because one edge can be both: a board small enough
    // puts a destination back on the very cell the wrap left from, and there
    // the exit's triangle has to win.
    let entries = wraps.iter().map(|&(entry, dir, _, progress)| {
        (
            (entry, dir),
            Mark {
                color: colors.at(progress),
                depth: 0.,
            },
        )
    });
    let exits = wraps.iter().map(|&(_, dir, destination, progress)| {
        (
            (destination, -dir),
            Mark {
                color: colors.at(progress),
                depth: progress,
            },
        )
    });
    entries.chain(exits).collect()
}

/// Every edge of a cell that wraps around the board, as `(cell, direction,
/// the cell it wraps to)`.
fn wrap_edges(board_dim: HexDim) -> impl Iterator<Item = (HexPoint, Dir, HexPoint)> {
    // a single step changes h and v by at most 1, so only the outermost rows and
    // columns can leave the board
    let on_border =
        move |pos: &HexPoint| pos.h == 0 || pos.v == 0 || pos.h == board_dim.h - 1 || pos.v == board_dim.v - 1;
    (0..board_dim.h)
        .flat_map(move |h| (0..board_dim.v).map(move |v| HexPoint { h, v }))
        .filter(on_border)
        .flat_map(move |pos| Dir::iter().filter_map(move |dir| Some((pos, dir, pos.wrap_destination(dir, board_dim)?))))
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;
    use crate::snake::{Segment, SegmentType};

    /// A teleport mark is painted over the border, so it has to ease back into
    /// what it covers: the border while it is drawn, the background once it is
    /// off. The outcome styles fade their own color out either way.
    #[test]
    fn a_teleport_mark_eases_into_what_it_covers() {
        let palette = Palette::dark();
        let mark = Mark { color: TINT.at(1.), depth: 1. };

        let with_border = absent_mark(HintStyle::Teleport, &palette, true, mark);
        assert_eq!(with_border.color, palette.border_color);
        assert_eq!(with_border.depth, 0., "and its triangle shrinks to nothing");

        let without = absent_mark(HintStyle::Teleport, &palette, false, mark);
        assert_eq!(without.color, palette.background_color);
        assert_eq!(without.color.a, 1., "it stays opaque, it is not a fade-out");

        for draw_border in [true, false] {
            let outcome = absent_mark(HintStyle::Border, &palette, draw_border, mark);
            assert_eq!(outcome.color.a, 0., "an outcome hint fades its own color out");
            assert_eq!(outcome.color.r, mark.color.r);
        }
    }

    /// Teleport hints are drawn on `(destination, -dir)`, which is only a
    /// border edge — and only meets the border cleanly — if wrapping is an
    /// involution: stepping back out of where you landed must leave the board
    /// again, and land back where you came from.
    #[test]
    fn wrapping_is_symmetric() {
        for board_dim in [
            HexPoint { h: 5, v: 5 },
            HexPoint { h: 10, v: 10 },
            HexPoint { h: 20, v: 13 },
            HexPoint { h: 7, v: 20 },
        ] {
            for (pos, dir, destination) in wrap_edges(board_dim) {
                let back = destination.wrap_destination(-dir, board_dim);
                assert_eq!(
                    back,
                    Some(pos),
                    "{board_dim:?}: {pos:?} {dir:?} -> {destination:?}, but back out is {back:?}",
                );
            }
        }
    }

    const BOARD: HexDim = HexPoint { h: 20, v: 20 };
    const BIG: HexDim = HexPoint { h: 80, v: 80 };
    const CELL_DIM: CellDim = CellDim { side: 50., sin: 43.30127, cos: 25. };
    const TINT: TeleportHintColors = TeleportHintColors {
        far_hue: 280.,
        near_hue: 0.,
        saturation: 0.636,
        lightness: 0.44,
    };

    /// A one-segment snake, its head `head_fraction` into `head` going `heading`.
    fn body_at(head: HexPoint, heading: Dir, head_fraction: f32) -> Body {
        Body {
            segments: VecDeque::from([Segment {
                segment_type: SegmentType::Normal,
                pos: head,
                coming_from: -heading,
                going_to: None,
                teleported: None,
                z_index: 0,
            }]),
            dir: heading,
            head_fraction,
            length: 1.,
            emerged: 1.,
            swallowed: 0.,
            length_changes: vec![],
            turn_start: None,
            search_trace: None,
        }
    }

    fn cell_center(cell: HexPoint) -> Point {
        cell.to_cartesian(CELL_DIM) + Hexagon::center(CELL_DIM)
    }

    /// Snapped to a cell center, each line runs out to the far side of the last
    /// on-board cell either way along its axis — the board's edge.
    #[test]
    fn line_hints_reach_the_border_both_ways() {
        let head = HexPoint { h: 3, v: 4 };
        let lines = line_hint_points(cell_center(head), head, BOARD, CELL_DIM);

        assert_eq!(lines.len(), 3, "one line per axis");
        for (line, dir) in lines.iter().zip([Dir::U, Dir::Ul, Dir::Ur]) {
            assert!(
                (line[1] - cell_center(head)).magnitude() < 0.01,
                "the line passes through the head's cell center",
            );

            for (end, dir) in [(line[0], dir), (line[2], -dir)] {
                // the last cell the line covers, found without stepping
                let mut last = head;
                while BOARD.contains(last.translate(dir, 1)) {
                    last = last.translate(dir, 1);
                }
                assert!(
                    !BOARD.contains(last.translate(dir, 1)),
                    "{last:?} should be the last cell going {dir:?}",
                );

                // half a cell past its center is the middle of its outer side
                let expected = cell_center(last) + board::cartesian_step(dir, CELL_DIM) * 0.5;
                assert!(
                    (end - expected).magnitude() < 0.01,
                    "the {dir:?} end should be at {expected:?}, got {end:?}",
                );
            }
        }
    }

    /// How far `point` is from the board's edge.
    fn distance_to_border(point: Point, board_dim: HexDim) -> f32 {
        border_segments(board_dim, CELL_DIM)
            .into_iter()
            .map(|(a, b, _)| {
                let along = b - a;
                let u = ((point - a).x * along.x + (point - a).y * along.y) / (along.x.powi(2) + along.y.powi(2));
                (point - (a + along * u.clamp(0., 1.))).magnitude()
            })
            .fold(f32::INFINITY, f32::min)
    }

    /// Wherever the head is within its cell, both ends of every line sit on the
    /// board's edge, ahead of the head along its own axis. From off the center
    /// that is no longer a cell side's middle: the ends slide along the edge.
    #[test]
    fn line_ends_stay_on_the_border_wherever_the_head_is() {
        for (h, v) in [
            (3, 4),
            (0, 0),
            (0, 9),
            (9, 0),
            (BOARD.h - 1, BOARD.v - 1),
            (5, BOARD.v - 1),
        ] {
            let head = HexPoint { h, v };
            for heading in Dir::iter() {
                for fraction in [0., 0.25, 0.5, 0.75, 1.] {
                    let origin = cell_center(head) + board::cartesian_step(heading, CELL_DIM) * (fraction - 0.5);
                    let lines = line_hint_points(origin, head, BOARD, CELL_DIM);

                    for (line, dir) in lines.iter().zip([Dir::U, Dir::Ul, Dir::Ur]) {
                        for (end, dir) in [(line[0], dir), (line[2], -dir)] {
                            let distance = distance_to_border(end, BOARD);
                            assert!(
                                distance < 0.01,
                                "{head:?} {heading:?} {fraction}: the {dir:?} end is {distance} off the border",
                            );

                            // and it is out along the axis, not behind the head
                            // (a head already on that border ends it right
                            // there, a line of no length)
                            let step = board::cartesian_step(dir, CELL_DIM);
                            let out = end - origin;
                            assert!(
                                (out.x * step.y - out.y * step.x).abs() < 0.01 * step.magnitude()
                                    && out.x * step.x + out.y * step.y >= 0.,
                                "{head:?} {heading:?} {fraction}: the {dir:?} end is not out along its axis",
                            );
                        }
                    }
                }
            }
        }
    }

    /// The smooth lines move with the head: half a step of travel moves the
    /// origin half a step, and the ends follow rather than staying put.
    #[test]
    fn smooth_lines_follow_the_head() {
        let head = HexPoint { h: 5, v: 5 };
        let at = |fraction: f32| {
            let origin = cell_center(head) + board::cartesian_step(Dir::Ur, CELL_DIM) * (fraction - 0.5);
            line_hint_points(origin, head, BOARD, CELL_DIM)
        };

        let (early, late) = (at(0.25), at(0.75));
        let travelled = (late[0][1] - early[0][1]).magnitude();
        let expected = board::cartesian_step(Dir::Ur, CELL_DIM).magnitude() / 2.;
        assert!(
            (travelled - expected).abs() < 0.01,
            "the origin should have moved {expected}, moved {travelled}",
        );

        // the U line is not the one being travelled along, so both its ends
        // have to slide along the border to keep up
        for end in [0, 2] {
            let moved = (late[0][end] - early[0][end]).magnitude();
            assert!(moved > 0.01, "the U line's end {end} did not move at all");
        }
    }

    /// A head about to cross marks the exit it is heading for at full depth —
    /// on the far side of the board, not under itself.
    #[test]
    fn a_head_about_to_cross_marks_its_exit_fully() {
        // at the left edge, heading off it, all but across
        let body = body_at(HexPoint { h: 0, v: 10 }, Dir::Ul, 1.);
        let hints = teleport_hints(&body, BOARD, TINT);

        let full: Vec<_> = hints.iter().filter(|(_, mark)| mark.depth > 0.999).collect();
        assert_eq!(full.len(), 1, "expected the exit straight ahead, got {full:?}");
        assert_eq!(full[0].0.0.h, BOARD.h - 1, "the exit is on the opposite edge");
        assert_eq!(full[0].0.1, -Dir::Ul, "and on the side it comes in through");
    }

    /// Both ends of a wrap sweep from purple to red in step with the triangle,
    /// and they always agree with each other.
    #[test]
    fn marks_sweep_from_purple_to_red_as_the_head_closes_in() {
        let hints_at = |v, fraction| teleport_hints(&body_at(HexPoint { h: 40, v }, Dir::U, fraction), BIG, TINT);

        // out at the edge of the range, the triangle not yet opening
        let far = hints_at(9, 0.);
        for mark in far.values() {
            assert_eq!(
                mark.color,
                TINT.at(0.),
                "out of triangle range it should be flat purple"
            );
            assert!(mark.color.b > mark.color.g, "purple is blue-ish: {:?}", mark.color);
        }

        // right up against one, all but across (the other directions off the
        // same border are wraps of their own, each at its own distance)
        let near = hints_at(0, 1.);
        let arriving = near
            .values()
            .max_by(|a, b| a.depth.total_cmp(&b.depth))
            .expect("the wrap straight ahead");
        assert_eq!(arriving.color, TINT.at(1.), "arriving it should be red");
        assert!(
            arriving.color.r > 2. * arriving.color.b,
            "red is red: {:?}",
            arriving.color
        );

        // and in between, both ends of a wrap share the triangle's progress
        let middle = hints_at(2, 0.);
        let depth = deepest(&middle);
        assert!(depth > 0. && depth < 1., "expected a part-open triangle, got {depth}");
        let entry = middle[&(HexPoint { h: 40, v: 0 }, Dir::U)];
        assert_eq!(entry.depth, 0., "the entry stays flat");
        assert_eq!(entry.color, TINT.at(depth), "but sweeps with its exit's triangle");
    }

    /// The deepest mark in `hints` — the exit's, entries always being flat.
    fn deepest(hints: &HashMap<(HexPoint, Dir), Mark>) -> f32 {
        hints.values().map(|mark| mark.depth).fold(0., f32::max)
    }

    /// Depth tracks the head continuously rather than a whole cell at a time —
    /// the point of measuring in fractions of a cell — and grows quadratically,
    /// so it creeps at first and rushes at the end.
    #[test]
    fn depth_follows_the_head_within_its_cell() {
        // 2 whole cells above it plus the rest of its own, so 3 away at the
        // start of the cell and 2 at the end
        let depth_at = |fraction| {
            let body = body_at(HexPoint { h: 40, v: 2 }, Dir::U, fraction);
            deepest(&teleport_hints(&body, BIG, TINT))
        };

        for (fraction, distance) in [(0., 3.), (0.25, 2.75), (0.5, 2.5), (1., 2.)] {
            let closeness: f32 = (TELEPORT_TRIANGLE_RANGE - distance) / TELEPORT_TRIANGLE_RANGE;
            let expected = closeness * closeness;
            let depth = depth_at(fraction);
            assert!(
                (depth - expected).abs() < 1e-5,
                "{fraction} into the cell should be {expected} deep, got {depth}",
            );
        }

        // half way along the growing stretch is a quarter of the way open
        let body = body_at(HexPoint { h: 40, v: 2 }, Dir::U, 0.5);
        assert!((deepest(&teleport_hints(&body, BIG, TINT)) - 0.25).abs() < 1e-5);
    }

    /// Between the two ranges both ends of the wrap are marked, and both flat.
    #[test]
    fn beyond_the_triangle_range_everything_is_flat() {
        // 7 away: inside TELEPORT_RANGE, outside TELEPORT_TRIANGLE_RANGE
        let body = body_at(HexPoint { h: 40, v: 6 }, Dir::U, 0.);
        let hints = teleport_hints(&body, BIG, TINT);

        assert_eq!(hints.len(), 2, "the entry and the exit, got {hints:?}");
        assert!(
            hints.values().all(|mark| mark.depth == 0. && mark.color.a == 1.),
            "everything out here is solid and flat: {hints:?}",
        );
    }

    /// Both ends of a wrap are marked, and they are two different edges.
    #[test]
    fn a_wrap_marks_the_border_it_leaves_and_the_one_it_arrives_at() {
        // straight up from near the top, far from every other border
        let body = body_at(HexPoint { h: 40, v: 2 }, Dir::U, 0.);
        let hints = teleport_hints(&body, BIG, TINT);

        let entry = hints[&(HexPoint { h: 40, v: 0 }, Dir::U)];
        assert_eq!(entry.depth, 0., "the border it leaves through stays flat");

        let exit = hints
            .iter()
            .find(|(_, mark)| mark.depth > 0.)
            .expect("an exit triangle");
        assert_eq!(exit.0.1, -Dir::U, "the exit is the side it arrives through");
        assert_ne!(
            exit.0.0,
            HexPoint { h: 40, v: 0 },
            "and a different cell on a big board"
        );
    }

    /// The far end of the range marks its border with no triangle at all, so a
    /// mark grows from nothing rather than appearing part-grown.
    #[test]
    fn the_edge_of_the_range_is_a_flat_mark() {
        // exactly TELEPORT_RANGE from the top, and far from every other border
        let body = body_at(HexPoint { h: 40, v: 9 }, Dir::U, 0.);
        let hints = teleport_hints(&body, BIG, TINT);

        assert_eq!(hints.len(), 2, "the entry and the exit, got {hints:?}");
        assert!(
            hints.values().all(|mark| mark.color.a == 1. && mark.depth.abs() < 1e-5),
            "still solid, but flat: {hints:?}",
        );
    }

    /// Which edges are marked must not change as the head crosses its cell.
    ///
    /// A corner reaches *four* wraps, not three: a diagonal through it is a
    /// single cell, so it leaves the board in both directions at once. Capping
    /// the count to three used to drop whichever sorted last, and since the
    /// distances shift with the head, the one dropped changed mid-cell and the
    /// mark flickered.
    ///
    /// Four wraps are eight marks, but a corner is also where an edge is both
    /// an entry and an exit, so several collapse into one.
    #[test]
    fn corner_wraps_are_all_marked_and_do_not_flicker() {
        let edges_at = |fraction| {
            let body = body_at(HexPoint { h: 0, v: 0 }, Dir::U, fraction);
            let mut edges: Vec<_> = teleport_hints(&body, BOARD, TINT).into_keys().collect();
            edges.sort_by_key(|&(cell, dir)| (cell, dir as u8));
            edges
        };

        let at_start = edges_at(0.);
        for fraction in [0.25, 0.5, 0.75, 0.99] {
            assert_eq!(
                edges_at(fraction),
                at_start,
                "the marked exits changed {fraction} of the way through the cell",
            );
        }
    }

    /// An edge that is both an entry and an exit keeps the exit's triangle.
    ///
    /// At a corner the diagonal through the board is one cell long, so going
    /// `Ur` out of `<0, 0>` arrives right back at `<0, 0>`: the `Dl` side of
    /// that cell is the entry for heading `Dl` and the exit for heading `Ur`
    /// at once. Entries are flat, so the exit has to be the one that survives.
    #[test]
    fn where_an_edge_is_both_the_exit_wins() {
        let body = body_at(HexPoint { h: 0, v: 0 }, Dir::U, 0.5);
        let hints = teleport_hints(&body, BOARD, TINT);

        let both = hints
            .get(&(HexPoint { h: 0, v: 0 }, Dir::Dl))
            .expect("the corner's Dl edge is marked");
        assert!(
            both.depth > 0.,
            "the exit's triangle should have won, got a flat mark: {hints:?}",
        );
    }

    /// Every mark is solid, and nothing out of range is marked at all.
    #[test]
    fn marks_are_solid_and_bounded_by_the_range() {
        for h in 0..BOARD.h {
            for v in 0..BOARD.v {
                let body = body_at(HexPoint { h, v }, Dir::U, 0.5);
                let hints = teleport_hints(&body, BOARD, TINT);
                // an entry and an exit per direction is the only bound there is
                assert!(hints.len() <= 12, "<{h}, {v}> marked {} edges", hints.len());
                assert!(hints.values().all(|mark| mark.color.a == 1. && mark.depth >= 0.));
            }
        }

        // the middle of a board far bigger than the range reaches nothing
        let body = body_at(HexPoint { h: 40, v: 40 }, Dir::U, 0.5);
        assert!(teleport_hints(&body, BIG, TINT).is_empty());
    }
}
