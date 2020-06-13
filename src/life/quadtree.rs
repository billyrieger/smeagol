// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    life::memory::Id,
    util::{Bit8x8, Grid2},
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Leaf {
    pub alive: Bit8x8,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Branch {
    pub children: Grid2<Id>,
    pub population: u128,
    pub level: u8,
}

/// This is documentation of an impl block. Why?
impl Node {
    pub fn level(&self) -> u8 {
        match self {
            Node::Leaf(_) => 3,
            Node::Branch(branch) => branch.level,
        }
    }
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
