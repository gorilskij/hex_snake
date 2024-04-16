use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::{Range, RangeInclusive};

use ggez::graphics::{Color, DrawMode, MeshBuilder};
use ggez::{mint, GameError, GameResult};
use itertools::{peek_nth, Itertools};

use crate::error::{Error, ErrorConversion, Result};
use crate::rendering::descriptions::RoundHeadDescription;
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
    segment_id: SegmentId,
    subsegment_idx: SubsegmentIdx,
}

struct Places {
    vertices: Range<usize>,
    indices: Range<usize>,
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

    fn contains_id(&self, id: SegmentId) -> bool {
        self.places.keys().any(|key| key.segment_id == id)
    }

    fn add_or_update_segment(&mut self, desc: SegmentDescription, color_resolution: ColorResolution) -> Result {
        desc.render(color_resolution)
            .try_for_each(|Polygon { subsegment_idx, points, color }| {
                match self.places.entry(SubsegmentKey {
                    segment_id: desc.segment_id,
                    subsegment_idx,
                }) {
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

    fn remove(builder: &mut MeshBuilder, places: &Places) {
        let _ = builder.buffer.vertices.drain(places.vertices.clone());
        let _ = builder.buffer.indices.drain(places.indices.clone());
    }

    fn remove_ids_lte(&mut self, id: SegmentId) {
        self.places.retain(|candidate_id, places| {
            if candidate_id.segment_id <= id {
                Self::remove(&mut self.inner, places);
                false
            } else {
                true
            }
        })
    }
}

pub struct SnakeCache {
    cached_builders: HashMap<ZIndex, Vec<BuilderBucket>>,
}

impl SnakeCache {
    const BUCKET_MAX_LEN: usize = 10;

    pub fn update(
        &mut self,
        segment_descriptions: impl Iterator<Item = SegmentDescription>,
        color_resolution: ColorResolution,
    ) -> Result {
        // TODO: iterator is __tail to head__

        let mut segment_descriptions = segment_descriptions.peekable();

        // build head
        let mut head_tail_builder = MeshBuilder::new();
        // TODO: return Err
        let tail = segment_descriptions.next().expect("iterator empty");

        // delete all segments that don't exist anymore, plus the segment that will be replaced by the tail
        self.cached_builders.retain(|_, buckets| {
            buckets.retain_mut(|bucket| {
                bucket.remove_ids_lte(tail.segment_id);
                !bucket.is_empty()
            });
            !buckets.is_empty()
        });

        // build tail
        // TODO: support different renderers
        SmoothSegments::render_segment(&tail, color_resolution).try_for_each(|polygon| {
            head_tail_builder
                .polygon(DrawMode::fill(), &polygon.points, *polygon.color)
                .map(|_| ())
        })?;

        // re-color existing segments
        while let Some(desc) = segment_descriptions.next() {
            if segment_descriptions.peek().is_none() {
                let head = desc;
                SmoothSegments::render_segment(&head, color_resolution).try_for_each(|polygon| {
                    head_tail_builder
                        .polygon(DrawMode::fill(), &polygon.points, *polygon.color)
                        .map(|_| ())
                })?;
            } else {
                let buckets = self
                    .cached_builders
                    .entry(desc.z_index)
                    .or_insert_with(|| Default::default());
                let mut bucket = buckets.iter_mut().find(|bucket| bucket.contains_id(desc.segment_id));
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

                bucket.add_or_update_segment(desc, color_resolution)?;

                // TODO #######################################
            }
        }

        // TODO: pop the old head, add the second element and then add the new head
        // TODO: for all existing segments, recolor
        // TODO: pop the old tail and (maybe, depending on eaten) the second-to-last segment and add the new tail
        // TODO: delete all segments that are *after* the last segment in the iterator
        // TODO: build everything

        // TODO: remember that segment HexPos locations can repeat
        todo!()
    }
}

// TODO: write z-index-aware draw method, or return multiple meshes or something
#[derive(Default)]
pub struct Cache {
    snake_caches: HashMap<SnakeUUID, SnakeCache>,
}
