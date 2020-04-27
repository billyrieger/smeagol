// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(incomplete_features)]
#![feature(const_generics, const_fn, maybe_uninit_slice_assume_init)]

pub mod bool8x8;
pub mod grid;
pub mod node;

use crate::bool8x8::Bool8x8;

/// A description of how one state of a cellular automaton transitions into the next.
#[derive(Clone, Copy, Debug)]
pub struct Rule {
    birth: [Bool8x8; 9],
    survival: [Bool8x8; 9],
}

impl Rule {
    /// Creates a new `Rule` in B/S notation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use smeagol::Rule;
    /// // Conway's Game of Life: B3/S23
    /// let life = Rule::new(&[3], &[2, 3]);
    /// ```
    ///
    /// [B/S notation]: https://www.conwaylife.com/wiki/Rulestring#B.2FS_notation
    pub fn new(birth: &[usize], survival: &[usize]) -> Self {
        Self {
            birth: Self::make_rule(birth),
            survival: Self::make_rule(survival),
        }
    }

    fn make_rule(neighbors: &[usize]) -> [Bool8x8; 9] {
        let mut result = [Bool8x8::FALSE; 9];
        for &i in neighbors.iter().filter(|&&i| i < 9) {
            result[i] = Bool8x8::TRUE;
        }
        result
    }
}
