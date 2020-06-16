// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    life::memory::{Id, Level},
    util::{BitSquare, Grid2},
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Cell {
    Dead,
    Alive,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Node<B> {
    Leaf(Leaf<B>),
    Branch(Branch),
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Leaf<B> {
    pub alive: B,
}

impl<B> Leaf<B>
where
    B: BitSquare,
{
    pub fn new(alive: B) -> Self {
        Self { alive }
    }

    pub fn dead() -> Self {
        Self::new(B::zero())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Branch {
    pub children: Grid2<Id>,
    pub population: u128,
    pub level: Level,
}

impl Branch {}

impl<B> Node<B>
where
    B: BitSquare,
{
    pub fn level(&self) -> Level {
        match self {
            Node::Leaf(_) => Level::new(B::LOG_SIDE_LEN),
            Node::Branch(branch) => branch.level,
        }
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
