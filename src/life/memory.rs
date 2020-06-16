// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    life::quadtree::{Branch, Leaf, Node},
    util::{BitSquare, Grid2},
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(pub u8);

impl Level {
    const MAX: Self = Level(63);
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id {
    level: Level,
    index: usize,
}

impl Id {
    const fn new(level: Level, index: usize) -> Self {
        Self { level, index }
    }
}

#[derive(Clone, Debug)]
pub struct Arena<B> {
    /// Why not use `[Vec<Node>; 64]`? Many traits in the standard library are only implemented for
    /// arrays up to an arbitrary maximum length. Check out [`std::array::LengthAtMost32`].
    buffers: [[Vec<Node<B>>; 32]; 2],
}

impl<B> Arena<B>
where
    B: BitSquare,
{
    pub fn new() -> Self {
        Self {
            buffers: Default::default(),
        }
    }

    pub fn init(&mut self) {
        let mut id = self.register(Node::Leaf(Leaf::dead()));
        let mut level = Level(B::LOG_SIDE_LEN);

        while level < Level::MAX {
            let branch = Branch {
                level,
                children: Grid2::repeat(id),
                population: 0,
            };

            id = self.register(Node::Branch(branch));

            level = Level(level.0 + 1);
        }
    }

    pub fn register(&mut self, node: Node<B>) -> Id {
        let level = node.level();

        let next_index = self.buffer(level).len();

        self.buffer_mut(level).push(node);

        Id::new(level, next_index)
    }

    pub fn empty_node(&self, level: Level) -> Node<B> {
        self.buffer(level)[0]
    }

    fn buffer(&self, level: Level) -> &[Node<B>] {
        let i = (level.0 / 32) as usize;
        let j = (level.0 % 32) as usize;
        &self.buffers[i][j]
    }

    fn buffer_mut(&mut self, level: Level) -> &mut Vec<Node<B>> {
        let i = (level.0 / 32) as usize;
        let j = (level.0 % 32) as usize;
        &mut self.buffers[i][j]
    }
}
