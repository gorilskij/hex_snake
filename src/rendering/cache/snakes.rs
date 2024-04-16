use std::collections::hash_map::Entry;
use std::collections::{BinaryHeap, HashMap};
use std::ops::Range;

use ggez::graphics::{DrawMode, Mesh, MeshBuilder};
use ggez::{Context, GameError};
use itertools::Itertools;

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

#[derive(Eq, PartialEq, Hash)]
struct SubsegmentKey {
    snake_uuid: SnakeUUID,
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

#[derive(Default)]
struct BuilderBucket {
    // id_range: IdRange,
    // cache blocks contain segments that all share a z-index
    inner: MeshBuilder,
    places: HashMap<SubsegmentKey, Places>,
}

impl BuilderBucket {
    fn new() -> Self {
        Self {
            inner: MeshBuilder::new(),
            places: Default::default(),
        }
    }

    fn is_empty(&self) -> bool {
        self.places.is_empty()
    }

    fn len(&self) -> usize {
        self.places.len()
    }

    fn contains(&self, snake_uuid: SnakeUUID, segment_id: SegmentId) -> bool {
        self.places
            .keys()
            .any(|key| key.snake_uuid == snake_uuid && key.segment_id == segment_id)
    }

    fn add_or_update_segment(
        &mut self,
        snake_uuid: SnakeUUID,
        desc: SegmentDescription,
        color_resolution: ColorResolution,
    ) -> Result {
        desc.render(color_resolution)
            .try_for_each(|Polygon { subsegment_idx, points, color, .. }| {
                let key = SubsegmentKey {
                    snake_uuid,
                    segment_id: desc.segment_id,
                    subsegment_idx,
                };

                match self.places.entry(key) {
                    Entry::Occupied(entry) => {
                        // assert_eq!(places.vertices.len(), 1);
                        entry
                            .get()
                            .vertices
                            .clone()
                            .into_iter()
                            .for_each(|i| self.inner.buffer.vertices[i].color = (*color).into())
                    }
                    Entry::Vacant(entry) => {
                        if points.len() >= 3 {
                            // polygons += 1
                            // builder.polygon(DrawMode::fill(), &points, *color).map(|_| ())

                            let orig_vertices_len = self.inner.buffer.vertices.len();
                            let orig_indices_len = self.inner.buffer.indices.len();
                            self.inner.polygon(DrawMode::fill(), &points, *color)?;
                            entry.insert(Places {
                                vertices: Range {
                                    start: orig_vertices_len,
                                    end: self.inner.buffer.vertices.len(),
                                },
                                indices: Range {
                                    start: orig_indices_len,
                                    end: self.inner.buffer.indices.len(),
                                },
                            });
                        } else {
                            // TODO: log, but actually make sure this doesn't happen
                            // Ok(())
                        }
                    }
                }
                Ok::<_, GameError>(())
            })
            .map_err(Error::from)
            .with_trace_step("BuilderBucket::add_or_update_segment")
    }

    fn remove_vertices(&mut self, range: Range<usize>) {
        let _ = self.inner.buffer.vertices.drain(range.clone());
        // correct other vertices
        self.places.iter_mut().for_each(|(_, places_to_correct)| {
            places_to_correct.correct_for_removal_of_vertices(range.clone())
        })
    }

    fn remove_indices(&mut self, range: Range<usize>) {
        let _ = self.inner.buffer.indices.drain(range.clone());
        // correct other indices
        self.places.iter_mut().for_each(|(_, places_to_correct)| {
            places_to_correct.correct_for_removal_of_indices(range.clone())
        })
    }

    fn remove_segment_ids_lte(&mut self, snake_uuid: SnakeUUID, segment_id: SegmentId) {
        println!(">>> remove_segment_ids_lte");
        // max-heaps
        let mut remove_vertices = BinaryHeap::new();
        let mut remove_indices = BinaryHeap::new();
        self.places.retain(|key, places| {
            !(key.snake_uuid == snake_uuid && key.segment_id <= segment_id) || {
                remove_vertices.push((places.vertices.start, places.vertices.end));
                remove_indices.push((places.indices.start, places.indices.end));
                false
            }
        });
        remove_vertices.into_iter().for_each(|(start, end)| self.remove_vertices(start..end));
        remove_indices.into_iter().for_each(|(start, end)| self.remove_indices(start..end));
    }

    fn remove_all(&mut self, snake_uuid: SnakeUUID) {
        println!(">>> remove_all");
        // max-heaps
        let mut remove_vertices = BinaryHeap::new();
        let mut remove_indices = BinaryHeap::new();
        self.places.retain(|key, places| {
            key.snake_uuid != snake_uuid || {
                remove_vertices.push((places.vertices.start, places.vertices.end));
                remove_indices.push((places.indices.start, places.indices.end));
                false
            }
        });
        remove_vertices.into_iter().for_each(|(start, end)| self.remove_vertices(start..end));
        remove_indices.into_iter().for_each(|(start, end)| self.remove_indices(start..end));
    }
}

#[derive(Default)]
pub struct Cache {
    head_tail_builders: HashMap<ZIndex, MeshBuilder>,
    cached_builders: HashMap<ZIndex, Vec<BuilderBucket>>,
    color_resolutions: HashMap<SnakeUUID, ColorResolution>,
}

impl Cache {
    // TODO: make variable
    // this is a soft limit in terms of subsegments,
    // segments are added one at a time so buckets can contain more
    // subsegments than this, however, if they do, no more segments
    // will be added
    const BUCKET_MAX_LEN: usize = 1;

    pub fn reset_head_tail_builder(&mut self) {
        self.head_tail_builders.clear()
    }

    pub fn update(
        &mut self,
        snake_uuid: SnakeUUID,
        // tail-to-head
        segment_descriptions: impl Iterator<Item = SegmentDescription>,
        color_resolution: ColorResolution,
    ) -> Result {
        let res: Result = try {
            // if a snake changes color resolution, its whole cache is invalidated
            if self
                .color_resolutions
                .get(&snake_uuid)
                .map(|res| res == &color_resolution)
                .unwrap_or(false)
            {
                self.cached_builders
                    .values_mut()
                    .for_each(|buckets| buckets.iter_mut().for_each(|bucket| bucket.remove_all(snake_uuid)));
            }

            let mut segment_descriptions = segment_descriptions.peekable();

            // TODO: return Err
            let tail = segment_descriptions.next().expect("iterator empty");

            // delete all segments that don't exist anymore, plus the segment that will be replaced by the tail
            self.cached_builders.retain(|_, buckets| {
                buckets.retain_mut(|bucket| {
                    bucket.remove_segment_ids_lte(snake_uuid, tail.segment_id);
                    !bucket.is_empty()
                });
                !buckets.is_empty()
            });

            // build tail
            // TODO: support different renderers
            println!("build tail");
            SmoothSegments::render_segment(&tail, color_resolution).try_for_each(|polygon| {
                self.head_tail_builders
                    .entry(tail.z_index)
                    .or_insert_with(|| MeshBuilder::new())
                    .polygon(DrawMode::fill(), &polygon.points, *polygon.color)
                    .map(|_| ())
            })?;
            println!("done");

            // re-color existing segments and build head
            while let Some(desc) = segment_descriptions.next() {
                if segment_descriptions.peek().is_none() {
                    let head = desc;
                    println!("build head");
                    SmoothSegments::render_segment(&head, color_resolution).try_for_each(|polygon| {
                        self.head_tail_builders
                            .entry(head.z_index)
                            .or_insert_with(|| MeshBuilder::new())
                            .polygon(DrawMode::fill(), &polygon.points, *polygon.color)
                            .map(|_| ())
                    })?;
                    println!("done");
                } else {
                    let buckets = self
                        .cached_builders
                        .entry(desc.z_index)
                        .or_insert_with(|| Default::default());
                    let mut bucket = buckets
                        .iter_mut()
                        .find(|bucket| bucket.contains(snake_uuid, desc.segment_id));
                    if bucket.is_none() {
                        // find the fullest bucket that isn't completely full
                        bucket = buckets
                            .iter_mut()
                            .filter(|bucket| bucket.len() < Self::BUCKET_MAX_LEN)
                            .max_by_key(|bucket| bucket.len());
                    }
                    if bucket.is_none() {
                        buckets.push(Default::default());
                        bucket = buckets.last_mut();
                    }
                    let bucket = bucket.unwrap();

                    bucket.add_or_update_segment(snake_uuid, desc, color_resolution)?;
                }
            }
        };

        res.with_trace_step("Cache::update")
    }

    // TODO: write z-index-aware draw method, or return multiple meshes or something
    pub fn build(&self, ctx: &Context) -> Vec<Mesh> {
        self.head_tail_builders
            .iter()
            .chain(
                self.cached_builders
                    .iter()
                    .flat_map(|(z_index, buckets)| buckets.iter().map(move |bucket| (z_index, &bucket.inner))),
            )
            .sorted_unstable_by_key(|(key, _)| *key)
            .map(|(_, builder)| Mesh::from_data(ctx, builder.build()))
            .collect()
    }
}
