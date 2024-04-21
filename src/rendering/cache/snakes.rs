use std::collections::hash_map::Entry;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::ops::Range;

use ggez::graphics::{DrawMode, Mesh, MeshBuilder};
use ggez::{Context, GameError};
use itertools::{peek_nth, Itertools};

use crate::error::{Error, ErrorConversion, Result};
use crate::rendering::point_factory::SegmentRenderer;
use crate::rendering::segments::descriptions::{Polygon, SegmentDescription};
use crate::rendering::segments::point_factory::ColorResolution;
use crate::rendering::segments::smooth_segments::{SmoothSegments, SubsegmentIdx};
use crate::snake::{SegmentId, SnakeUUID, ZIndex};

// TODO: track number of polygons created

// INVARIANT: self.0.start <= self.0.end
// #[repr(transparent)]
// struct IdRange(RangeInclusive<SegmentId>);

// impl IdRange {
//     fn all_ids_lt(&self, id: SegmentId) -> bool {
//         self.0.end() <= &id
//     }
// }

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct SubsegmentKey {
    segment_id: SegmentId,
    subsegment_idx: SubsegmentIdx,
}

#[derive(Clone)]
struct Places {
    vertices: Range<usize>,
    indices: Range<usize>,
}

impl Places {
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

struct BuilderBucket {
    z_index: ZIndex,
    places: HashMap<SubsegmentKey, Places>,
    inner: MeshBuilder,
}

impl BuilderBucket {
    fn new(z_index: ZIndex) -> Self {
        Self {
            z_index,
            places: Default::default(),
            inner: MeshBuilder::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.places.is_empty()
    }

    fn len(&self) -> usize {
        self.places.len()
    }

    fn contains(&self, segment_id: SegmentId) -> bool {
        self.places.keys().any(|key| key.segment_id == segment_id)
    }

    fn build(inner: &mut MeshBuilder, polygon: Polygon) -> Result<Option<Places>> {
        if polygon.points.len() >= 3 {
            // polygons += 1

            // debug
            let orig_vertices = inner.buffer.vertices.clone();
            let orig_indices = inner.buffer.indices.clone();

            // build polygon
            let orig_vertices_len = inner.buffer.vertices.len();
            let orig_indices_len = inner.buffer.indices.len();
            inner.polygon(DrawMode::fill(), &polygon.points, *polygon.color)?;

            debug_assert_eq!(&orig_vertices, &inner.buffer.vertices[..orig_vertices_len]);
            debug_assert_eq!(&orig_indices, &inner.buffer.indices[..orig_indices_len]);

            Ok(Some(Places {
                vertices: orig_vertices_len..inner.buffer.vertices.len(),
                indices: orig_indices_len..inner.buffer.indices.len(),
            }))
        } else {
            // TODO: log, but actually make sure this doesn't happen
            Ok(None)
        }
    }

    fn add_or_update_segment(&mut self, desc: SegmentDescription, color_resolution: ColorResolution) -> Result {
        desc.render(color_resolution)
            .try_for_each(|polygon| {
                let key = SubsegmentKey {
                    segment_id: desc.segment_id,
                    subsegment_idx: polygon.subsegment_idx,
                };

                match self.places.entry(key) {
                    Entry::Occupied(entry) => {
                        // recolor
                        entry
                            .get()
                            .vertices
                            .clone()
                            .into_iter()
                            .for_each(|i| self.inner.buffer.vertices[i].color = (*polygon.color).into())
                    }
                    Entry::Vacant(entry) => {
                        if let Some(places) = Self::build(&mut self.inner, polygon)? {
                            entry.insert(places);
                        }
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
const BUCKET_MAX_LEN: usize = 50;

struct SnakeCache {
    // if the color resolution changes, the cache is invalidated
    color_resolution: ColorResolution,
    buckets: Vec<BuilderBucket>,
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
        segment_descriptions: impl Iterator<Item = SegmentDescription> + Clone,
    ) -> Result {
        println!("##### UPDATE, num_buckets: {}", self.buckets.len());
        println!(
            "{:?}",
            segment_descriptions.clone().map(|desc| desc.segment_id).collect_vec()
        );
        // println!(
        //     "{:?}",
        //     self.buckets
        //         .iter()
        //         .map(|bucket| {
        //             let x: Box<dyn Iterator<Item = SegmentId>> = match &bucket.state {
        //                 BucketState::Cached { places } => Box::new(places.iter().map(|(key, _)| key.segment_id)),
        //                 BucketState::Dying { keys } => Box::new(keys.iter().map(|key| key.segment_id)),
        //             };
        //             x.collect::<HashSet<_>>()
        //         })
        //         .collect_vec()
        // );

        let res: Result = try {
            // TODO: have a mechanism to prevent color_resolution from changing too often
            //       (make the increase threshold higher than the decrease threshold)
            //       but either this needs to be done above the cache, or the c_r calculation
            //           code needs to be moved into the cache
            if color_resolution != self.color_resolution {
                // invalidate the cache
                self.buckets.clear();
            }

            let mut segment_descriptions = peek_nth(segment_descriptions);
            // let mut segment_descriptions = segment_descriptions.peekable();

            // TODO: return Err
            let tail = segment_descriptions.next().expect("iterator empty");

            // delete buckets that contain segments that don't exist anymore plus buckets that contain the tail segment
            self.buckets
                .retain_mut(|bucket| bucket.places.keys().all(|key| key.segment_id > tail.segment_id));

            // build tail
            // TODO: support different renderers
            let builder = head_tail_builders
                .entry(tail.z_index)
                .or_insert_with(|| MeshBuilder::new());
            SmoothSegments::render_segment(&tail, color_resolution).try_for_each(|polygon| {
                builder
                    .polygon(DrawMode::fill(), &polygon.points, *polygon.color)
                    .map(|_| ())
            })?;

            // re-color existing segments and build head
            while let Some(desc) = segment_descriptions.next() {
                // always redraw the first two segments as both can contain parts of the round head
                // always redraw trailing segments whose bucket has been deleted
                if self.trailing_segments.contains(&desc.segment_id) || segment_descriptions.peek_nth(1).is_none() {
                    let builder = head_tail_builders
                        .entry(desc.z_index)
                        .or_insert_with(|| MeshBuilder::new());
                    SmoothSegments::render_segment(&desc, color_resolution).try_for_each(|polygon| {
                        builder
                            .polygon(DrawMode::fill(), &polygon.points, *polygon.color)
                            .map(|_| ())
                    })?;
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
                            self.buckets.push(BuilderBucket::new(desc.z_index));
                            self.buckets.last_mut().unwrap()
                        }
                    };

                    bucket.add_or_update_segment(desc, color_resolution)?;
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
    pub fn reset_head_tail_builder(&mut self) {
        self.head_tail_builders.clear()
    }

    pub fn update(
        &mut self,
        snake_uuid: SnakeUUID,
        color_resolution: ColorResolution,
        // tail-to-head
        segment_descriptions: impl Iterator<Item = SegmentDescription> + Clone, // TODO: remove Clone
    ) -> Result {
        self.snake_caches
            .entry(snake_uuid)
            .or_insert_with(|| SnakeCache::new(color_resolution))
            .update(&mut self.head_tail_builders, color_resolution, segment_descriptions)
            .with_trace_step("Cache::update")
    }

    pub fn build(&self, ctx: &Context) -> Vec<Mesh> {
        self.head_tail_builders
            .iter()
            .chain(self.snake_caches.iter().flat_map(|(_, snake_cache)| {
                snake_cache
                    .buckets
                    .iter()
                    .map(|bucket| (&bucket.z_index, &bucket.inner))
            }))
            .sorted_unstable_by_key(|(z_index, _)| *z_index)
            .map(|(_, builder)| Mesh::from_data(ctx, builder.build()))
            .collect()
    }
}
