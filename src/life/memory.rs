// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    life::quadtree::{Branch, Leaf, Node},
    util::{BitSquare, Grid2},
    Error, Result,
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level {
    index: u8,
}

impl Level {
    const MAX: Level = Level::new(63);

    pub const fn new(index: u8) -> Self {
        Self { index }
    }
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

pub struct Buffer<T> {
    buf: Vec<Option<T>>,
    next_free: usize,
}

impl<T> Buffer<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            // every slot in the buffer is initially empty
            buf: std::iter::repeat_with(|| None).take(capacity).collect(),
            next_free: 0,
        }
    }

    pub fn try_insert(&mut self, value: T) -> Result<usize> {
        self.buf
            .get_mut(self.next_free)
            .ok_or(Error)?
            .replace(value);

        let value_index = self.next_free;

        while let Some(Some(_)) = self.buf.get(self.next_free) {
            self.next_free += 1;
        }

        Ok(value_index)
    }
}

#[derive(Clone, Debug)]
pub struct Arena<B> {
    /// Why not use `[Vec<Node>; 64]`? Many traits in the standard library, such as `Debug` and
    /// `Default`, are only implemented for arrays up to an arbitrary maximum length. Check out
    /// [`std::array::LengthAtMost32`].
    buffers: [[Vec<Node<B>>; 32]; 2],
}

impl<B> Default for Arena<B> {
    fn default() -> Self {
        Self {
            buffers: Default::default(),
        }
    }
}

impl<B> Arena<B>
where
    B: BitSquare,
{
    pub fn new() -> Self {
        let mut result = Self::default();

        result.init();

        result
    }

    fn init(&mut self) {
        let mut id = self.register(Node::Leaf(Leaf::dead()));
        let mut level = Level::new(B::LOG_SIDE_LEN);

        while level < Level::MAX {
            let branch = Branch {
                level,
                children: Grid2::repeat(id),
                population: 0,
            };

            id = self.register(Node::Branch(branch));

            level = Level::new(level.index + 1);
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
        let i = (level.index / 32) as usize;
        let j = (level.index % 32) as usize;
        &self.buffers[i][j]
    }

    fn buffer_mut(&mut self, level: Level) -> &mut Vec<Node<B>> {
        let i = (level.index / 32) as usize;
        let j = (level.index % 32) as usize;
        &mut self.buffers[i][j]
    }
}
