// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(const_fn, const_if_match)]
#![allow(dead_code, unused_variables)]

pub mod bool8x8;

use bool8x8::Bool8x8;

const fn make_rule(neighbors: &[u8]) -> [Bool8x8; 9] {
    match neighbors {
        [] => [Bool8x8::FALSE; 9],
        [head, tail @ ..] => {
            let mut result = make_rule(tail);
            result[*head as usize] = Bool8x8::TRUE;
            result
        }
    }
}

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
    /// const LIFE: Rule = Rule::new(&[3], &[2, 3]);
    /// ```
    pub const fn new(birth: &[u8], survival: &[u8]) -> Self {
        Self {
            birth: make_rule(birth),
            survival: make_rule(survival),
        }
    }
}
