// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(dead_code, unused_variables)]

mod node;
mod rle;
mod store;
mod util;

use std::fmt;
use store::{Id, Store};
use util::{Bool8x8, Offset, SumResult};

use thiserror::Error;

/// A result.
pub type Result<T> = std::result::Result<T, Error>;

/// A runtime error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("parse")]
    RleParse,
    #[error("increment")]
    Increment,
    #[error("unbalanced")]
    Unbalanced,
    #[error("unbalanced")]
    IdNotFound(Id),
    #[error("out of bounds")]
    OutOfBounds,
    #[error("fmt {0}")]
    Fmt(#[from] fmt::Error),
}

/// The smallest and fundamental element of a cellular automaton.
///
/// A cell can have one of two states: `Cell::Dead` and `Cell::Alive`.
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum Cell {
    Dead,
    Alive,
}

impl Cell {
    /// ```
    /// # use smeagol::Cell;
    /// assert!(Cell::Alive.is_alive());
    /// assert!(!Cell::Dead.is_alive());
    /// ```
    pub fn is_alive(&self) -> bool {
        match self {
            Cell::Dead => false,
            Cell::Alive => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    /// Creates a new `Position` from the given `x` and `y` coordinates.
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn offset(&self, dx: i64, dy: i64) -> Position {
        Self::new(self.x + dx, self.y + dy)
    }
}

/// A description of how one state of a cellular automaton transitions into the next.
#[derive(Clone, Copy, Debug, Default)]
pub struct Rule {
    birth_neighbors: SumResult,
    survival_neighbors: SumResult,
}

impl Rule {
    /// Creates a new `Rule` using B/S notation.
    ///
    /// From [LifeWiki]:
    ///
    /// > The most common notation for rulestrings B{number list}/S{number list}; this is referred
    /// > to as "B/S notation", and is sometimes called the rulestring of the [cellular automaton]
    /// > in question. B (for birth) is a list of all the numbers of live neighbors that cause a
    /// > dead cell to come alive (be born); S (for survival) is a list of all the numbers of live
    /// > neighbors that cause a live cell to remain alive (survive).
    ///
    /// [LifeWiki]: https://www.conwaylife.com/wiki/Rulestring#B.2FS_notation
    ///
    /// # Examples
    ///
    /// ```
    /// # use smeagol::Rule;
    /// // Conway's Game of Life: B3/S23
    /// let life = Rule::new(&[3], &[2, 3]);
    /// ```
    pub fn new(birth: &[usize], survival: &[usize]) -> Self {
        let make_rule = |neighbor_count: &[usize]| -> SumResult {
            let mut result = [Bool8x8::FALSE; 9];
            for &i in neighbor_count.iter().filter(|&&count| count < 9) {
                result[i] = Bool8x8::TRUE;
            }
            result
        };

        Self {
            birth_neighbors: make_rule(birth),
            survival_neighbors: make_rule(survival),
        }
    }
}

pub struct Universe {
    store: Store,
    root: Id,
}

impl Universe {
    pub fn empty(rule: Rule) -> Result<Self> {
        let mut store = Store::new(rule);
        let root = store.initialize()?;
        Ok(Self { store, root })
    }

    /// # Examples
    ///
    /// ```
    /// # use smeagol::{Rule, Universe};
    /// let universe = Universe::empty(Rule::new(&[3], &[2, 3]));
    /// ```
    pub fn get_cell(&self, x: i64, y: i64) -> Result<Cell> {
        self.store.get_cell(self.root, x, y)
    }

    pub fn set_cell(&mut self, x: i64, y: i64, cell: Cell) -> Result<()> {
        self.root = self.store.set_cell(self.root, x, y, cell)?;
        Ok(())
    }

    pub fn step(&mut self, steps: u64) {
        todo!()
    }
}
