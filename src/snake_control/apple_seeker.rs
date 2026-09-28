use std::collections::HashSet;

use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::{Dir, HexDim, HexPoint};
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::{Body, SegmentType};
use crate::snake_control::Controller;
use crate::snake_control::appetite::Appetite;
use crate::snake_control::pathfinder::{
    Committed, Goals, Leg, Obstacle, Obstacles, Path, PathFinder, Plan, surroundings,
};
use crate::view::snakes::Snakes;

/// More targets than this is a planning mistake, not a configuration: each one
/// is another search, and a route that long is stale before it is walked.
const MAX_TARGETS: usize = 8;

/// Seeks apples by planning a route through the next few of them (using the
/// wrapped [`PathFinder`] strategy one leg at a time) and following it.
///
/// The plan is kept as long as it stays true: the head walks it off the front,
/// and a new leg is appended at the back as targets are eaten, so the part of
/// the route already committed to never moves. Only something actually going
/// wrong — the board resizing, an obstacle appearing, a target vanishing,
/// straying off the path — throws any of it away.
///
/// A fallback (the survival crawl, when no target can be reached) is walked
/// the same way, and only given up for a real plan once one may be possible
/// (see [`Opening`]).
pub struct AppleSeeker {
    pub pathfinder: Box<dyn PathFinder + Send + Sync>,
    /// How many targets to plan ahead for.
    pub targets: usize,
    /// Which apples are targets and which are to be avoided.
    pub appetite: Appetite,
    pub plan: Option<Plan>,
    /// While the plan is a fallback, what would make a target reachable.
    pub opening: Option<Opening>,
    /// The cells to avoid as of the last tick, so that one appearing on the
    /// route since is noticed (once: the route may be the best there is).
    pub known_avoid: HashSet<HexPoint>,
}

/// Where the snake could go when it fell back to crawling: no target could be
/// reached, so one only can be once a cell walling it in frees up, or a target
/// appears in the part of the board it can reach.
pub struct Opening {
    region: HashSet<HexPoint>,
    walls: HashSet<HexPoint>,
    /// Targets already in the region, which it couldn't get to anyway (facing
    /// the wrong way, say): only a new one is worth another try.
    targets: HashSet<HexPoint>,
}

impl Opening {
    fn new(head: HexPoint, targets: &[HexPoint], board_dim: HexDim, blocks: impl Fn(HexPoint) -> bool) -> Self {
        let (region, walls) = surroundings(head, board_dim, blocks);
        let targets = targets.iter().copied().filter(|pos| region.contains(pos)).collect();
        Self { region, walls, targets }
    }

    /// Whether a target may have become reachable since.
    fn opened(&self, targets: &[HexPoint], blocks: impl Fn(HexPoint) -> bool) -> bool {
        self.walls.iter().any(|&wall| !blocks(wall))
            || targets
                .iter()
                .any(|pos| self.region.contains(pos) && !self.targets.contains(pos))
    }
}

impl AppleSeeker {
    fn recalculate_plan(
        &mut self,
        body: &Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
    ) {
        debug_assert!(self.targets <= MAX_TARGETS, "{} targets is too many", self.targets);
        let head = body.segments[0].pos;
        let goals = self.appetite.goals(apples);
        let known_avoid = std::mem::replace(&mut self.known_avoid, goals.avoid.keys().copied().collect());

        // keep whatever is still true
        if let Some(plan) = &mut self.plan {
            let obstacles = Obstacles::new(body, knowledge, other_snakes);
            let mut keep = plan.board_dim == gtx.board_dim && follow(plan, head);
            if keep {
                drop_spent_legs(plan);
                keep = connects(plan, gtx.board_dim)
                    && !blocked_ahead(plan, &obstacles)
                    && !newly_avoided_ahead(plan, &goals, &known_avoid);
            }
            if keep && plan.is_fallback() {
                keep = self
                    .opening
                    .as_ref()
                    .is_some_and(|opening| !opening.opened(&goals.targets, |pos| obstacles.blocks(pos)));
            }
            if keep {
                // a target someone else ate (or one that expired) ends the plan
                // there; the legs before it are still good
                if let Some(idx) = plan
                    .legs
                    .iter()
                    .position(|leg| leg.target.is_some_and(|target| !goals.targets.contains(&target)))
                {
                    plan.legs.truncate(idx);
                }
                promote_targets_on_route(plan, &goals.targets);
            } else {
                self.plan = None;
            }
        }

        // and extend it to the full number of targets
        let mut plan = self.plan.take().unwrap_or_else(|| Plan {
            legs: vec![],
            board_dim: gtx.board_dim,
        });
        // a fallback goes nowhere in particular, so there is nothing to extend
        while !plan.is_fallback() && plan.legs.len() < self.targets {
            // each leg starts where the previous one ends, the first at the head
            let start = match plan.legs.last() {
                Some(leg) => match leg.arrival(gtx.board_dim) {
                    Some(start) => start,
                    None => break,
                },
                None => (head, body.dir),
            };

            let targets: Vec<HexPoint> = plan.targets().collect();
            let cells: Vec<(HexPoint, Obstacle)> = committed_cells(&plan, &targets, knowledge);
            let committed = Committed { cells: &cells, targets: &targets };

            let leg = self
                .pathfinder
                .get_path(start, &goals, committed, body, knowledge, other_snakes, gtx);

            match leg {
                // A leg going nowhere in particular is as far as planning goes,
                // and only worth having at all when there is nothing else: tack
                // it onto a good plan and the whole plan counts as a fallback,
                // walked only until a real plan may be possible.
                Some(leg) if leg.target.is_none() => {
                    if plan.legs.is_empty() {
                        plan.legs.push(leg);
                        let obstacles = Obstacles::new(body, knowledge, other_snakes);
                        let blocks = |pos| obstacles.blocks(pos);
                        self.opening = Some(Opening::new(head, &goals.targets, gtx.board_dim, blocks));
                    }
                    break;
                }
                Some(leg) => plan.legs.push(leg),
                None => break,
            }
        }

        if plan.legs.is_empty() {
            println!("failed to find path");
            println!("apples: {}", apples.len());
        }
        if !plan.is_fallback() {
            self.opening = None;
        }
        self.plan = (!plan.legs.is_empty()).then_some(plan);
    }
}

/// What the route already commits to, as the next leg's search will find it.
///
/// A target's cell will hold an eaten segment by the time the route comes back
/// around, which the snake may be able to pass through; everywhere else it will
/// be body, which it may not.
fn committed_cells(plan: &Plan, targets: &[HexPoint], knowledge: Option<&Knowledge>) -> Vec<(HexPoint, Obstacle)> {
    let eaten = SegmentType::Eaten { original_food: 1., food_left: 1. };
    let pass_eaten = knowledge.is_some_and(|knowledge| knowledge.can_pass_through_own(eaten));

    plan.cells()
        .map(|pos| {
            let obstacle = if pass_eaten && targets.contains(&pos) {
                Obstacle::Passable
            } else {
                Obstacle::Blocked
            };
            (pos, obstacle)
        })
        .collect()
}

/// Make a target of every wanted apple that has appeared on the route.
///
/// The snake is going to walk over it and eat it either way, so the plan should
/// say so: the leg it lands on splits in two at that cell, which leaves the
/// route itself untouched. This is what can push a plan past the number of
/// targets it plans for — extending simply waits until it is back under.
fn promote_targets_on_route(plan: &mut Plan, targets: &[HexPoint]) {
    let claimed: Vec<HexPoint> = plan.targets().collect();

    let mut idx = 0;
    while idx < plan.legs.len() {
        let cells = &plan.legs[idx].cells;
        // the first cell is the head or the previous leg's target, and the last
        // is this leg's own; neither is new
        let found = (1..cells.len().saturating_sub(1))
            .find(|&at| targets.contains(&cells[at]) && !claimed.contains(&cells[at]));

        let Some(at) = found else {
            idx += 1;
            continue;
        };

        // the two legs share the cell they meet in, like any other pair
        let leg = &mut plan.legs[idx];
        let rest: Path = leg.cells.iter().skip(at).copied().collect();
        let target = leg.cells[at];
        leg.cells.truncate(at + 1);
        let tail = Leg { cells: rest, target: leg.target };
        leg.target = Some(target);
        plan.legs.insert(idx + 1, tail);
        idx += 1;
    }
}

/// Walk the plan forward to wherever the head is now, dropping legs it has
/// finished. `false` if the head isn't on the plan at all any more.
fn follow(plan: &mut Plan, head: HexPoint) -> bool {
    loop {
        let Some(leg) = plan.legs.first_mut() else { return false };

        if leg.cells.front() == Some(&head) {
            return true;
        }
        if leg.cells.len() >= 2 && leg.cells[1] == head {
            leg.cells.pop_front();
            return true;
        }
        // Nothing left of this leg for the head to be on: its target has been
        // reached, and the next leg starts in that same cell. Otherwise the
        // head is somewhere else entirely and the plan is worthless.
        if leg.cells.len() < 2 {
            plan.legs.remove(0);
        } else {
            return false;
        }
    }
}

/// Drop a leading leg the head has nothing left to walk, so the leg being
/// followed always has the next step in it. The leg after it starts in the same
/// cell, so nothing is lost — but a spent leg left in front would make the
/// snake hold its course through a cell it should be turning in.
fn drop_spent_legs(plan: &mut Plan) {
    while plan.legs.len() > 1 && plan.legs[0].cells.len() < 2 {
        plan.legs.remove(0);
    }
}

/// Whether the next step is one a snake can actually take. A freshly planned
/// leg always connects; one the head was moved along by other means may not.
fn connects(plan: &Plan, board_dim: HexDim) -> bool {
    let Some(leg) = plan.legs.first() else { return false };
    leg.cells.len() < 2 || leg.cells[0].single_step_dir_to(leg.cells[1], board_dim).is_some()
}

/// Whether anything has moved into the way of the rest of the plan.
fn blocked_ahead(plan: &Plan, obstacles: &Obstacles) -> bool {
    // the head itself is the first cell
    plan.cells().skip(1).any(|pos| obstacles.blocks(pos))
}

/// Whether something to avoid has appeared on the rest of the route since the
/// last tick. The route is worth another look then, but only then: if it is
/// still the best way, it stays.
fn newly_avoided_ahead(plan: &Plan, goals: &Goals, known_avoid: &HashSet<HexPoint>) -> bool {
    plan.cells()
        .skip(1)
        .any(|pos| goals.avoid.contains_key(&pos) && !known_avoid.contains(&pos))
}

impl Controller for AppleSeeker {
    fn next_dir(
        &mut self,
        body: &mut Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
    ) -> Option<Dir> {
        self.recalculate_plan(body, knowledge, other_snakes, apples, gtx);

        // the first leg with a step left in it — `drop_spent_legs` keeps that
        // one at the front, finding it is belt and braces
        let leg: Option<&Leg> = self.plan.as_ref()?.legs.iter().find(|leg| leg.cells.len() >= 2);

        match leg {
            // not `dir_to`: across the board's edge that points the opposite
            // way. A freshly (re)calculated plan always connects.
            Some(leg) => leg.cells[0].single_step_dir_to(leg.cells[1], gtx.board_dim),
            // nothing left to walk: about to eat the last target, hold course
            None => Some(body.dir),
        }
    }

    fn get_plan(
        &mut self,
        body: &Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        apples: &[Apple],
        gtx: &GameContext,
    ) -> Option<&Plan> {
        self.recalculate_plan(body, knowledge, other_snakes, apples, gtx);
        self.plan.as_ref()
    }

    fn reset(&mut self, _dir: Dir) {
        self.plan = None;
        self.opening = None;
        self.known_avoid.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic::HexDim;

    const BOARD: HexDim = HexPoint { h: 20, v: 20 };

    fn cell(h: isize) -> HexPoint {
        HexPoint { h, v: 5 }
    }

    /// One leg running along a row, from `from` to `to`, its target the last cell.
    fn leg(from: isize, to: isize) -> Leg {
        Leg {
            cells: (from..=to).map(cell).collect(),
            target: Some(cell(to)),
        }
    }

    fn plan(legs: Vec<Leg>) -> Plan {
        Plan { legs, board_dim: BOARD }
    }

    /// An apple that appears on the route is walked over and eaten either way,
    /// so the plan splits there and counts it as a target. The route itself
    /// must come out exactly as it went in.
    #[test]
    fn an_apple_on_the_route_becomes_a_target() {
        let mut route = plan(vec![leg(0, 6)]);
        let before: Vec<HexPoint> = route.cells().collect();

        promote_targets_on_route(&mut route, &[cell(3)]);

        assert_eq!(route.legs.len(), 2, "the leg should have split at the apple");
        assert_eq!(route.legs[0].target, Some(cell(3)), "the new target");
        assert_eq!(route.legs[1].target, Some(cell(6)), "and the old one after it");
        assert_eq!(
            route.cells().collect::<Vec<_>>(),
            before,
            "splitting a leg must not move the route",
        );
        // the two legs meet in the target's cell, like any other pair
        assert_eq!(route.legs[0].cells.back(), route.legs[1].cells.front());
    }

    /// Several apples on one leg, and apples that are already targets, are all
    /// accounted for — the second split has to be found in the tail of the
    /// first one.
    #[test]
    fn every_apple_on_the_route_is_promoted_once() {
        let mut route = plan(vec![leg(0, 8)]);

        promote_targets_on_route(&mut route, &[cell(2), cell(5), cell(8)]);

        let targets: Vec<HexPoint> = route.targets().collect();
        assert_eq!(targets, vec![cell(2), cell(5), cell(8)]);

        // running it again finds nothing new
        let legs = route.legs.len();
        promote_targets_on_route(&mut route, &[cell(2), cell(5), cell(8)]);
        assert_eq!(route.legs.len(), legs, "targets are not promoted twice");
    }

    /// Something to avoid landing on the route is worth one replan, but a
    /// route that is still the best way is kept after that.
    #[test]
    fn a_bad_apple_on_the_route_is_noticed_once() {
        let route = plan(vec![leg(0, 6)]);
        let goals = Goals {
            targets: vec![cell(6)],
            avoid: std::collections::HashMap::from([(cell(3), 15)]),
        };
        assert!(newly_avoided_ahead(&route, &goals, &HashSet::new()));
        assert!(!newly_avoided_ahead(&route, &goals, &HashSet::from([cell(3)])));
    }

    /// A pocket of cells along a row, walled in by everything else.
    fn pocket(cells: std::ops::RangeInclusive<isize>) -> HashSet<HexPoint> {
        cells.map(cell).collect()
    }

    #[test]
    fn a_crawl_is_kept_while_the_pocket_stays_shut() {
        let open = pocket(0..=4);
        let opening = Opening::new(cell(2), &[], BOARD, |pos| !open.contains(&pos));
        assert!(!opening.opened(&[], |pos| !open.contains(&pos)));
        // an apple outside the pocket is still out of reach
        assert!(!opening.opened(&[cell(8)], |pos| !open.contains(&pos)));
    }

    #[test]
    fn a_wall_freeing_up_is_worth_another_try() {
        let open = pocket(0..=4);
        let opening = Opening::new(cell(2), &[], BOARD, |pos| !open.contains(&pos));
        let wider = pocket(0..=5);
        assert!(opening.opened(&[], |pos| !wider.contains(&pos)));
    }

    #[test]
    fn only_a_new_apple_in_the_pocket_is_worth_another_try() {
        let open = pocket(0..=4);
        let blocks = |pos| !open.contains(&pos);
        // there already, and it couldn't be reached then
        let opening = Opening::new(cell(2), &[cell(4)], BOARD, blocks);
        assert!(!opening.opened(&[cell(4)], blocks));
        assert!(opening.opened(&[cell(4), cell(1)], blocks));
    }

    /// The route is in the way of whatever is planned after it, except where it
    /// will have eaten something and can pass through — and that last part is
    /// the snake's own eat mechanics talking, not an assumption.
    #[test]
    fn the_route_blocks_what_comes_after_it() {
        let route = plan(vec![leg(0, 4)]);
        let targets: Vec<HexPoint> = route.targets().collect();

        let passes = Knowledge::always(true);
        let committed = committed_cells(&route, &targets, Some(&passes));
        assert_eq!(committed.len(), 5, "every cell of the route: {committed:?}");
        assert_eq!(
            committed.iter().find(|&&(pos, _)| pos == cell(4)).unwrap().1,
            Obstacle::Passable,
            "the target will be an eaten segment, which this snake passes through",
        );
        assert!(
            committed
                .iter()
                .filter(|&&(pos, _)| pos != cell(4))
                .all(|&(_, o)| o == Obstacle::Blocked),
            "the rest of the route is body: {committed:?}",
        );

        let crashes = Knowledge::always(false);
        let committed = committed_cells(&route, &targets, Some(&crashes));
        assert!(
            committed.iter().all(|&(_, obstacle)| obstacle == Obstacle::Blocked),
            "a snake that crashes into its own eaten segments has to go around: {committed:?}",
        );
    }
}
