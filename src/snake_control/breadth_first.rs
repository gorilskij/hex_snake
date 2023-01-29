use crate::basic::{Dir, HexDim, HexPoint};

use crate::snake::eat_mechanics::Knowledge;
use std::rc::Rc;

pub struct BreadthFirst {
    pub passthrough_knowledge: Knowledge,
    pub target: Option<HexPoint>,
    pub path: Vec<Dir>,
    pub steps_since_update: usize,
}

struct PathNode {
    /// Where the path reaches
    point: HexPoint,
    length: usize,
    parent: Option<Rc<Self>>,
}

impl PathNode {
    // Directions to take to follow this path
    fn to_dir_vec(&self, board_dim: HexDim) -> Vec<Dir> {
        let mut vec = vec![Dir::U; self.length];
        // fill vec in reverse order
        let mut former = Some(self);
        for i in (0..self.length).rev() {
            let latter = former;
            former = former.and_then(|n| n.parent.as_ref().map(Rc::as_ref));
            vec[i] = former
                .unwrap()
                .point
                .wrapping_dir_to_1(latter.unwrap().point, board_dim)
                .unwrap()
        }

        assert_eq!(vec.len(), self.length);
        vec
    }
}

impl BreadthFirst {
    fn recalculate_path(&mut self) {}
}

// impl Controller for BreadthFirst {
//     fn next_dir(
//         &mut self,
//         body: &mut Body,
//         other_snakes: OtherSnakes,
//         apples: &[Apple],
//         gtx: &GameContext,
//         _ctx: &Context,
//     ) -> Option<Dir> {
//
//     }
// }
