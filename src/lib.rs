// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(const_fn, const_if_match)]

mod bool8x8;

pub mod grid;
pub mod node;
pub mod store;

use node::{Id, Level};

use thiserror::Error;

pub use bool8x8::*;

/// A runtime error.
#[derive(Debug, Error)]
pub enum Error {
    /// The step sisze was too large.
    #[error("step size {step:?} too large for node with level {level:?}")]
    StepOverflow { step: u64, level: Level },

    /// Increment error.
    #[error("cannot increment past the maximum level")]
    Increment,

    /// Id not found.
    #[error("id {0:?} not found")]
    IdNotFound(Id),

    /// Unbalanced.
    #[error("unbalanced")]
    Unbalanced,
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
    pub const fn new(birth: &[usize], survival: &[usize]) -> Self {
        let empty = [Bool8x8::FALSE; 9];
        Self {
            birth_neighbors: Self::make_rule(empty, birth),
            survival_neighbors: Self::make_rule(empty, survival),
        }
    }

    /// Evolves a `Bool8x8` to its next state, treating `true` as alive and `false` as dead.
    pub const fn step(&self, cells: Bool8x8) -> Bool8x8 {
        let (alive, dead) = (cells, cells.not());

        let (n, s) = (alive.offset(0, 1), alive.offset(0, -1));
        let (e, w) = (alive.offset(1, 0), alive.offset(-1, 0));
        let (ne, nw) = (alive.offset(1, 1), alive.offset(-1, 1));
        let (se, sw) = (alive.offset(1, -1), alive.offset(-1, -1));
        let alive_neighbors = Bool8x8::sum(&[nw, n, ne, w, e, sw, s, se]);

        let born = Bool8x8::any_both(&alive_neighbors, &self.birth_neighbors);
        let survives = Bool8x8::any_both(&alive_neighbors, &self.survival_neighbors);

        dead.and(born).or(alive.and(survives))
    }

    const fn make_rule(result: SumResult, neighbors: &[usize]) -> SumResult {
        match neighbors {
            [] => result,
            &[head, ref tail @ ..] => {
                let mut result = result;
                if head < result.len() {
                    result[head] = Bool8x8::TRUE;
                }
                Self::make_rule(result, tail)
            }
        }
    }
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
