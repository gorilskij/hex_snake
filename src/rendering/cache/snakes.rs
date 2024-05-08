use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::mem::size_of;
use std::ops::Range;

use ggez::graphics::{DrawMode, LinearColor, Mesh, MeshBuilder, MeshData, Vertex};
use ggez::Context;
use itertools::{peek_nth, Itertools};

use crate::app::stats::Stats;
use crate::error::{ErrorConversion, Result};
use crate::rendering::segments::descriptions::{Polygon, SegmentDescription};
use crate::rendering::segments::point_factory::ColorResolution;
use crate::rendering::segments::smooth_segments::SubsegmentIdx;
use crate::snake::{SegmentId, SnakeUUID, ZIndex};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct SubsegmentKey {
    segment_id: SegmentId,
    subsegment_idx: SubsegmentIdx,
}

#[derive(Clone)]
struct Place {
    vertices: Range<usize>,
    indices: Range<usize>,
}

impl Place {
    fn correct_for_removal_of_vertices(&mut self, removed: Range<usize>) {
        // assert that the ranges don't overlap
        assert!(self.vertices.end <= removed.start || removed.end <= self.vertices.start);

        if removed.end <= self.vertices.start {
            self.vertices.start -= removed.len();
            self.vertices.end -= removed.len();
        }
    }

    fn correct_for_removal_of_indices(&mut self, removed: Range<usize>) {
        // assert that the ranges don't overlap
        assert!(self.indices.end <= removed.start || removed.end <= self.indices.start);

        if removed.end <= self.indices.start {
            self.indices.start -= removed.len();
            self.indices.end -= removed.len();
        }
    }
}

#[derive(Default)]
struct Mirror {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

enum Inner {
    Filling { builder: MeshBuilder, mirror: Mirror },
    Full(Mesh),
}

struct MemoryMesh {
    z_index: ZIndex,
    inner: Inner,
    places: HashMap<SubsegmentKey, Place>,
}

impl MemoryMesh {
    fn new(z_index: ZIndex) -> Self {
        Self {
            z_index,
            inner: Inner::Filling {
                builder: MeshBuilder::new(),
                mirror: Default::default(),
            },
            places: Default::default(),
        }
    }
}

impl MemoryMesh {
    fn len(&self) -> usize {
        self.places.len()
    }

    fn is_full(&self) -> bool {
        matches!(&self.inner, Inner::Full(_))
    }

    fn contains(&self, segment_id: SegmentId) -> bool {
        self.places.keys().any(|key| key.segment_id == segment_id)
    }

    fn build(builder: &mut MeshBuilder, mirror: &mut Mirror, polygon: Polygon) -> Result<Place> {
        // build polygon
        let orig_vertices_len = mirror.vertices.len();
        let orig_indices_len = mirror.indices.len();
        builder.polygon(DrawMode::fill(), &polygon.points, *polygon.color)?;
        let new_data = builder.build();
        mirror
            .vertices
            .extend(new_data.vertices[orig_vertices_len..].iter().copied());
        mirror
            .indices
            .extend(new_data.indices[orig_indices_len..].iter().copied());

        Ok(Place {
            vertices: orig_vertices_len..mirror.vertices.len(),
            indices: orig_indices_len..mirror.indices.len(),
        })
    }

    fn mark_full(&mut self, ctx: &Context) {
        match &mut self.inner {
            Inner::Filling { builder, .. } => {
                self.inner = Inner::Full(Mesh::from_data(ctx, builder.build()));
            }
            Inner::Full(_) => panic!("mark_full called on a full mesh"),
        }
    }

    fn add_or_recolor_segment(
        &mut self,
        desc: SegmentDescription,
        color_resolution: ColorResolution,
        stats: &mut Stats,
    ) -> Result {
        desc.render(color_resolution)
            .try_for_each(|polygon| {
                let key = SubsegmentKey {
                    segment_id: desc.segment_id,
                    subsegment_idx: polygon.subsegment_idx,
                };

                match self.places.entry(key) {
                    Entry::Occupied(entry) => {
                        // recolor
                        let Inner::Full(mesh) = &mut self.inner else {
                            panic!("subsegment exists but inner is Filling");
                        };

                        entry.get().vertices.clone().into_iter().for_each(|i| {
                            let len = mesh.vertex_count() * size_of::<Vertex>();
                            let verts = &mut mesh.verts.slice(..len as u64).get_mapped_range_mut() as &mut [_];
                            let verts: &mut [Vertex] = bytemuck::cast_slice_mut(verts);
                            verts[i].color = LinearColor::from(*polygon.color).into()
                        })
                    }
                    Entry::Vacant(entry) => {
                        // add new segments to the builder
                        let Inner::Filling { builder, mirror } = &mut self.inner else {
                            panic!("subsegment doesn't exist but inner is Full");
                        };

                        let places = Self::build(builder, mirror, polygon)?;
                        stats.polygons += 1;
                        entry.insert(places);
                    }
                }

                Ok(())
            })
            .with_trace_step("BuilderBucket::add_or_update_segment")
    }
}

type HeadTailBuilders = HashMap<ZIndex, MeshBuilder>;

// TODO: make variable
// this is a soft limit in terms of subsegments,
// segments are added one at a time so buckets can contain more
// subsegments than this, however, if they do, no more segments
// will be added
// const BUCKET_MAX_LEN: usize = 200;
const BUCKET_MAX_LEN: usize = 20;

struct SnakeCache {
    // if the color resolution changes, the cache is invalidated
    color_resolution: ColorResolution,
    buckets: Vec<MemoryMesh>,
    trailing_segments: HashSet<SegmentId>,
}

impl SnakeCache {
    fn new(color_resolution: ColorResolution) -> Self {
        Self {
            color_resolution,
            buckets: vec![],
            trailing_segments: Default::default(),
        }
    }

    fn update(
        &mut self,
        head_tail_builders: &mut HeadTailBuilders,
        color_resolution: ColorResolution,
        // tail-to-head
        segment_descriptions: impl Iterator<Item = SegmentDescription>,
        stats: &mut Stats,
    ) -> Result {
        let res: Result = try {
            // TODO: have a mechanism to prevent color_resolution from changing too often
            //       (make the increase threshold higher than the decrease threshold)
            //       but either this needs to be done above the cache, or the c_r calculation
            //           code needs to be moved into the cache
            if color_resolution != self.color_resolution {
                // invalidate the cache
                self.buckets.clear();
                self.color_resolution = color_resolution;
            }

            let mut segment_descriptions = peek_nth(segment_descriptions);
            // let mut segment_descriptions = segment_descriptions.peekable();

            let tail = segment_descriptions.next().expect("iterator empty");

            // delete buckets that contain segments that don't exist anymore plus buckets that contain the tail segment
            self.buckets
                .retain_mut(|bucket| bucket.places.keys().all(|key| key.segment_id > tail.segment_id));

            // build tail
            let builder = head_tail_builders
                .entry(tail.z_index)
                .or_insert_with(|| MeshBuilder::new());
            stats.polygons += tail.build(builder, color_resolution)?;

            // re-color existing segments and build head
            while let Some(desc) = segment_descriptions.next() {
                // always redraw the first two segments as both can contain parts of the round head
                // always redraw trailing segments whose bucket has been deleted
                if self.trailing_segments.contains(&desc.segment_id) || segment_descriptions.peek_nth(1).is_none() {
                    let builder = head_tail_builders
                        .entry(desc.z_index)
                        .or_insert_with(|| MeshBuilder::new());
                    stats.polygons += desc.build(builder, color_resolution)?;
                } else {
                    let bucket = {
                        let bucket = self.buckets.iter_mut().find(|bucket| {
                            bucket.contains(desc.segment_id) || {
                                bucket.z_index == desc.z_index && bucket.places.len() < BUCKET_MAX_LEN
                            }
                        });
                        if let Some(bucket) = bucket {
                            bucket
                        } else {
                            self.buckets.push(MemoryMesh::new(desc.z_index));
                            self.buckets.last_mut().unwrap()
                        }
                    };

                    bucket.add_or_recolor_segment(desc, color_resolution, stats)?;
                }
            }
        };
        res.with_trace_step("SnakeCache::update")
    }
}

#[derive(Default)]
pub struct Cache {
    head_tail_builders: HeadTailBuilders,
    snake_caches: HashMap<SnakeUUID, SnakeCache>,
}

impl Cache {
    pub fn clear(&mut self) {
        self.snake_caches.clear();
    }

    pub fn build_frame(&mut self) -> FrameBuilder {
        FrameBuilder(self)
    }
}

pub struct FrameBuilder<'a>(&'a mut Cache);

impl FrameBuilder<'_> {
    pub fn update(
        &mut self,
        snake_uuid: SnakeUUID,
        color_resolution: ColorResolution,
        // tail-to-head
        segment_descriptions: impl Iterator<Item = SegmentDescription>,
        stats: &mut Stats,
    ) -> Result {
        self.0
            .snake_caches
            .entry(snake_uuid)
            .or_insert_with(|| SnakeCache::new(color_resolution))
            .update(
                &mut self.0.head_tail_builders,
                color_resolution,
                segment_descriptions,
                stats,
            )
            .with_trace_step("Cache::update")
    }

    pub fn build(self, ctx: &Context) -> Vec<Mesh> {
        self.0.snake_caches.values_mut().flat_map(|snake_cache| {
           snake_cache.buckets.iter_mut()
        }).for_each(|bucket| {
            if bucket.len() >= BUCKET_MAX_LEN {
                bucket.mark_full(ctx);
            }
        });

        let meshes = self
            .0
            .head_tail_builders
            .iter()
            .map(|(z_index, builder)| (*z_index, Mesh::from_data(ctx, builder.build())))
            .chain(self.0.snake_caches.values().flat_map(|snake_cache| {
                snake_cache
                    .buckets
                    .iter()
                    .map(|bucket| match &bucket.inner {
                        Inner::Filling { builder, .. } => (bucket.z_index, Mesh::from_data(ctx, builder.build())),
                        Inner::Full(mesh) => (bucket.z_index, mesh.clone()),
                    })
            }))
            .sorted_unstable_by_key(|(z_index, _)| *z_index)
            .map(|(_, mesh)| mesh)
            .collect();

        self.0.head_tail_builders.clear();
        meshes
    }
}
