// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    life::quadtree::{Branch, Leaf, Node},
    util::Grid2,
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id {
    level: u8,
    index: usize,
}

impl Id {
    const fn new(level: u8, index: usize) -> Self {
        Self { level, index }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Arena {
    // Why not use `[Vec<Node>; 64]`? Many traits in the standard library are only implemented for
    // arrays up to an arbitrary maximum length. Check out `std::array::LengthAtMost32`.
    buffers: [[Vec<Node>; 32]; 2],
}

impl Arena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self) {
        let mut empty: Node = Node::Leaf(Leaf::dead());
        let mut current_level = empty.level();

        let empty_id: Id = Id::new(empty.level(), 0);
        let no_children: Grid2<Id> = Grid2::repeat(empty_id);

        for buf in self.buffers.iter_mut().flatten() {
            let next_branch = Branch {
                level: current_level,
                children: no_children,
                population: 0,
            };

            empty = Node::Branch(next_branch);
            current_level += 1;
            buf.push(empty);
        }
    }

    pub fn create_node(&mut self, node: Node) -> Id {
        let level: u8 = node.level();

        let (i, j) = Self::level_to_indices(level);

        let buflen = self.buffers[i][j].len();

        let new_id = Id::new(level, buflen);

        self.buffers[i][j].push(node);

        new_id
    }

    fn level_to_indices(level: u8) -> (usize, usize) {
        assert!(level < 64);
        let level = level as usize;
        (level / 32, level % 32)
    }
}
