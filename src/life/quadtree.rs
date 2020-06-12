// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    util::{Bit8x8, Grid2},
    Error, Result,
};

use std::{
    convert::TryFrom,
    ops::{Index, IndexMut},
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub const ORIGIN: Position = Self::new(0, 0);

    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub const fn offset(&self, dx: i64, dy: i64) -> Position {
        Self::new(self.x + dx, self.y + dy)
    }

    pub const fn relative_to(&self, other: Position) -> Position {
        self.offset(-other.x, -other.y)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

/// This is documentation of an impl block. Why?
impl Node {
    pub fn level(&self) -> Level {
        match self {
            Node::Leaf(_) => Level(3),
            Node::Branch(branch) => branch.level,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Leaf {
    pub alive: Bit8x8,
}

impl Leaf {
    pub const fn new(alive: Bit8x8) -> Self {
        Self { alive }
    }

    pub const fn dead() -> Self {
        Self::new(Bit8x8(0))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Branch {
    pub level: Level,
    pub children: Grid2<Id>,
    pub population: u128,
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
            current_level = Level(current_level.0 + 1);
            buf.push(empty);
        }
    }

    pub fn create_node(&mut self, node: Node) -> Id {
        let buf: &mut Vec<Node> = &mut self[node.level()];
        let new_id = Id::new(node.level(), buf.len());
        buf.push(node);
        new_id
    }

    fn level_indices(level: u8) -> (usize, usize) {
        assert!(level < 64);
        let level = level as usize;
        (level / 32, level % 32)
    }
}

impl Index<Level> for Arena {
    type Output = Vec<Node>;

    fn index(&self, idx: Level) -> &Vec<Node> {
        assert!(idx.0 < 64);
        let (i, j) = Self::level_indices(idx.0);
        &self.buffers[i][j]
    }
}

impl IndexMut<Level> for Arena {
    fn index_mut(&mut self, idx: Level) -> &mut Vec<Node> {
        assert!(idx.0 < 64);
        let (i, j) = Self::level_indices(idx.0);
        &mut self.buffers[i][j]
    }
}
