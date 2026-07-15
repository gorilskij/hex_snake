//! Dynamic wormholes: pairs of linked portal cells that open at random spots
//! on the board, stay open for a while, then collapse and reopen elsewhere.
//!
//! Built on alt portals: each wormhole is a pair of `alt::Portal::cell`s, one
//! for each direction of travel. The system keeps a fixed population of
//! wormholes open, replacing each one some seconds after it opens.

use std::time::Duration;

use rand::distributions::uniform::SampleRange;
use rand::Rng;

use crate::app::portal::alt;
use crate::app::screen::Environment;
use crate::basic::board::get_occupied_cells;
use crate::basic::HexPoint;

struct Wormhole {
    a: HexPoint,
    b: HexPoint,
    /// Time left until this wormhole collapses
    ttl: Duration,
}

pub struct Wormholes {
    holes: Vec<Wormhole>,
}

impl Wormholes {
    /// How many wormholes are kept open at a time
    const COUNT: usize = 2;
    /// Mouths only open at least this far from the board edge so that both
    /// the mouth and all its exit cells stay on the board
    const BORDER_MARGIN: isize = 2;
    /// Minimum distance between the two mouths of a wormhole (in cells),
    /// also kept from the mouths of other wormholes
    const MIN_SEPARATION: usize = 8;
    /// Lifetime range of a wormhole (seconds)
    const LIFETIME: std::ops::Range<f32> = 8.0..16.0;
    /// How many positions to try before giving up until the next update
    const ATTEMPTS: usize = 50;

    pub fn new() -> Self {
        Self { holes: vec![] }
    }

    /// Collapse all wormholes (on restart or board resize); the population is
    /// re-established on the next update
    pub fn clear<Rng>(&mut self, env: &mut Environment<Rng>) {
        self.holes.clear();
        env.alt_portals.clear();
    }

    /// Tick lifetimes, collapse expired wormholes, and open new ones to
    /// maintain the population. Returns whether the set of wormholes changed
    /// (meaning the portal mesh must be rebuilt).
    pub fn update<R: Rng>(&mut self, env: &mut Environment<R>, elapsed: Duration) -> bool {
        let mut changed = false;

        // tick down and collapse
        let before = self.holes.len();
        self.holes.retain_mut(|hole| match hole.ttl.checked_sub(elapsed) {
            Some(ttl) => {
                hole.ttl = ttl;
                true
            }
            None => false,
        });
        changed |= self.holes.len() != before;

        // open new wormholes to maintain the population
        while self.holes.len() < Self::COUNT {
            match self.open(env) {
                Some(hole) => {
                    self.holes.push(hole);
                    changed = true;
                }
                // no space found, try again on a later update
                None => break,
            }
        }

        if changed {
            env.alt_portals = self
                .holes
                .iter()
                .flat_map(|hole| {
                    [
                        alt::Portal::cell(hole.a, hole.b),
                        alt::Portal::cell(hole.b, hole.a),
                    ]
                })
                .map(|mut portal| {
                    portal.update(env.gtx.board_dim);
                    portal
                })
                .collect();
        }

        changed
    }

    /// Try to find positions for a new wormhole
    fn open<R: Rng>(&self, env: &mut Environment<R>) -> Option<Wormhole> {
        let board_dim = env.gtx.board_dim;

        // board too small for the margin: no wormholes
        if board_dim.h <= 2 * Self::BORDER_MARGIN + 1 || board_dim.v <= 2 * Self::BORDER_MARGIN + 1 {
            return None;
        }

        // includes the dead cells of currently open wormholes
        let occupied = get_occupied_cells(&env.snakes, &env.apples, &env.alt_portals);

        let sample_mouth = |rng: &mut R| -> Option<HexPoint> {
            let pos = HexPoint {
                h: (Self::BORDER_MARGIN..board_dim.h - Self::BORDER_MARGIN).sample_single(rng),
                v: (Self::BORDER_MARGIN..board_dim.v - Self::BORDER_MARGIN).sample_single(rng),
            };

            // the mouth cell and all its exit cells must be free
            let blocked = occupied.binary_search(&pos).is_ok();
            // keep clear of the mouths of other wormholes
            let too_close = self
                .holes
                .iter()
                .flat_map(|hole| [hole.a, hole.b])
                .any(|mouth| mouth.manhattan_distance(pos) < Self::MIN_SEPARATION);

            (!blocked && !too_close).then_some(pos)
        };

        for _ in 0..Self::ATTEMPTS {
            let Some(a) = sample_mouth(&mut env.rng) else { continue };
            let Some(b) = sample_mouth(&mut env.rng) else { continue };

            if a.manhattan_distance(b) < Self::MIN_SEPARATION {
                continue;
            }

            let ttl = Duration::from_secs_f32(Self::LIFETIME.sample_single(&mut env.rng));
            return Some(Wormhole { a, b, ttl });
        }

        None
    }
}
