// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(const_fn, const_if_match)]
#![allow(dead_code, unused_variables)]

pub mod leaf;
// pub mod node;

use leaf::Bool8x8;

#[derive(Clone, Copy, Debug)]
pub struct Rule {
    birth: [Bool8x8; 9],
    survival: [Bool8x8; 9],
}

impl Rule {
    /// Creates a new Life-like rule in B/S notation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use smeagol::Rule;
    /// // Conway's Game of Life: B3/S23
    /// const LIFE: Rule = Rule::new(&[3], &[2, 3]);
    /// ```
    pub const fn new(birth: &[u8], survival: &[u8]) -> Self {
        Self {
            birth: make_rule(birth),
            survival: make_rule(survival),
        }
    }
}

const fn make_rule(neighbors: &[u8]) -> [Bool8x8; 9] {
    match neighbors {
        [] => [Bool8x8::FALSE; 9],
        [head, tail @ ..] => {
            let [r0, r1, r2, r3, r4, r5, r6, r7, r8] = make_rule(tail);
            let t_ = Bool8x8::TRUE;
            match head {
                0 => [t_, r1, r2, r3, r4, r5, r6, r7, r8],
                1 => [r0, t_, r2, r3, r4, r5, r6, r7, r8],
                2 => [r0, r1, t_, r3, r4, r5, r6, r7, r8],
                3 => [r0, r1, r2, t_, r4, r5, r6, r7, r8],
                4 => [r0, r1, r2, r3, t_, r5, r6, r7, r8],
                5 => [r0, r1, r2, r3, r4, t_, r6, r7, r8],
                6 => [r0, r1, r2, r3, r4, r5, t_, r7, r8],
                7 => [r0, r1, r2, r3, r4, r5, r6, t_, r8],
                8 => [r0, r1, r2, r3, r4, r5, r6, r7, t_],
                _ => [r0, r1, r2, r3, r4, r5, r6, r7, r8],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_rule() {
        let empty = [Bool8x8::FALSE; 9];
        assert_eq!(make_rule(&[]), empty);
        assert_eq!(make_rule(&[9]), empty);
        assert_eq!(
            make_rule(&[8, 8, 1, 8, 3, 100, 3, 1, 33]),
            make_rule(&[8, 3, 1])
        );
    }
}
