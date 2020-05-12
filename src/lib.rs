// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(dead_code, unused_variables)]

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod node;
mod rle;
mod util;

use node::{Branch, Id, Node, Store};
use util::{Bool8x8, Offset, SumResult};

use thiserror::Error;

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
    IdNotFound(crate::node::Id),
}

/// A result.
pub type Result<T> = std::result::Result<T, Error>;

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

    /// Evolves a `Bool8x8` to its next state, treating `true` as alive and `false` as dead.
    fn step(&self, cells: Bool8x8) -> Bool8x8 {
        let (alive, dead) = (cells, !cells);

        let alive_neighbors: SumResult = Bool8x8::sum(&[
            alive.offset(Offset::West(1)),
            alive.offset(Offset::East(1)),
            alive.offset(Offset::North(1)),
            alive.offset(Offset::South(1)),
            alive.offset(Offset::Northwest(1)),
            alive.offset(Offset::Northeast(1)),
            alive.offset(Offset::Southwest(1)),
            alive.offset(Offset::Southeast(1)),
        ]);

        let any_both = |xs: &SumResult, ys: &SumResult| -> Bool8x8 {
            xs.iter()
                .zip(ys.iter())
                .map(|(&x, &y)| x & y)
                .fold(Bool8x8::FALSE, std::ops::BitOr::bitor)
        };

        let born = any_both(&alive_neighbors, &self.birth_neighbors);
        let survives = any_both(&alive_neighbors, &self.survival_neighbors);

        (dead & born) | (alive & survives)
    }
}

pub struct Universe {
    rule: Rule,
    store: Store,
    root: Id,
}

#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum Cell {
    Dead,
    Alive,
}

impl Universe {
    pub fn get_cell(&self, x: i64, y: i64) -> Cell {
        todo!()
    }

    pub fn set_cell(&mut self, x: i64, y: i64) {
        todo!()
    }

    pub fn unset_cell(&mut self, x: i64, y: i64) {
        todo!()
    }

    pub fn toggle_cell(&mut self, x: i64, y: i64) {
        todo!()
    }

    pub fn step(&mut self, steps: u64) {
        todo!()
    }
}

pub struct Region {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blinker() {
        let life = Rule::new(&[3], &[2, 3]);

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x38 | . . # # # . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let blinker_horiz = Bool8x8(0x0000_0038_0000_0000);

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x10 | . . . # . . . .
        // 0x10 | . . . # . . . .
        // 0x10 | . . . # . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let blinker_vert = Bool8x8(0x0000_1010_1000_0000);

        assert_eq!(life.step(blinker_horiz), blinker_vert);
        assert_eq!(life.step(blinker_vert), blinker_horiz);
    }
}
