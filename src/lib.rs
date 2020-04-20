// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(const_fn, const_if_match)]

mod counter;

use counter::Counter;

/// Conway's Game of Life.
pub const LIFE: Rule = Rule::new(&[3], &[2, 3]);

/// An eight by eight grid of cells.
///
/// ```txt
/// 
///      100…000
///      MOST SIGNIFICANT BIT
///       v
///     +---+---+---+---+---+---+---+---+
///     | 63| 62| 61| 60| 59| 58| 57| 56|
///     +---+---+---+---+---+---+---+---+
///     | 55| 54| 53| 52| 51| 50| 49| 48|
///     +---+---+---+---+---+---+---+---+
///     | 47| 46| 45| 44| 43| 42| 41| 40|
///     +---+---+---+---+---+---+---+---+
///     | 39| 38| 37| 36| 35| 34| 33| 32|
///     +---+---+---+---+---+---+---+---+
///     | 31| 30| 29| 28| 27| 26| 25| 24|
///     +---+---+---+---+---+---+---+---+
///     | 23| 22| 21| 20| 19| 18| 17| 16|
///     +---+---+---+---+---+---+---+---+
///     | 15| 14| 13| 12| 11| 10|  9|  8|
///     +---+---+---+---+---+---+---+---+
///     |  7|  6|  5|  4|  3|  2|  1|  0|
///     +---+---+---+---+---+---+---+---+
///                                   ^
///                LEAST SIGNIFICANT BIT
///                              000…001
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Leaf(pub u64);

impl Leaf {
    /// Creates a `Leaf` from a grid of cells represented by a `[u8; 8]`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use smeagol::Leaf;
    /// let glider = Leaf::from_array([
    ///     0b_00000000,
    ///     0b_00000000,
    ///     0b_00001000,
    ///     0b_00000100,
    ///     0b_00011100,
    ///     0b_00000000,
    ///     0b_00000000,
    ///     0b_00000000,
    /// ]);
    /// ```
    pub const fn from_array(cells: [u8; 8]) -> Self {
        Self(u64::from_be_bytes(cells))
    }

    const fn shift(&self, dx: i8, dy: i8) -> Self {
        let mut result = *self;
        result.0 = if dx < 0 {
            result.0 << (-dx) as u8
        } else {
            result.0 >> dx as u8
        };
        result.0 = if dy < 0 {
            result.0 >> (-dy * 8) as u8
        } else {
            result.0 << (dy * 8) as u8
        };
        result
    }

    pub const fn tick(&self, rule: &Rule) -> Self {
        rule.step(*self)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rule {
    birth: [u64; 9],
    survival: [u64; 9],
}

impl Rule {
    const fn make_rule(neighbors: &[u8]) -> [u64; 9] {
        match neighbors {
            &[] => [u64::MIN; 9],
            &[head, ref tail @ ..] => {
                let mut result = Self::make_rule(tail);
                result[head as usize] = u64::MAX;
                result
            }
        }
    }

    /// # Examples
    ///
    /// ```
    /// # use smeagol::Rule;
    /// let conways_game_of_life = Rule::new(&[3], &[2, 3]);
    /// ```
    pub const fn new(birth: &[u8], survival: &[u8]) -> Self {
        Self {
            birth: Self::make_rule(birth),
            survival: Self::make_rule(survival),
        }
    }

    pub const fn step(&self, leaf: Leaf) -> Leaf {
        let birth = self.birth;
        let survival = self.survival;

        let sums = Counter::new()
            .add(leaf.shift(0, 1).0)
            .add(leaf.shift(0, -1).0)
            .add(leaf.shift(1, 0).0)
            .add(leaf.shift(-1, 0).0)
            .add(leaf.shift(1, 1).0)
            .add(leaf.shift(-1, -1).0)
            .add(leaf.shift(1, -1).0)
            .add(leaf.shift(-1, 1).0)
            .finish();

        let alive = leaf.0;
        let dead = !leaf.0;
        let result = u64::MIN
            | sums[0] & (dead & birth[0] | alive & survival[0])
            | sums[1] & (dead & birth[1] | alive & survival[1])
            | sums[2] & (dead & birth[2] | alive & survival[2])
            | sums[3] & (dead & birth[3] | alive & survival[3])
            | sums[4] & (dead & birth[4] | alive & survival[4])
            | sums[5] & (dead & birth[5] | alive & survival[5])
            | sums[6] & (dead & birth[6] | alive & survival[6])
            | sums[7] & (dead & birth[7] | alive & survival[7])
            | sums[8] & (dead & birth[8] | alive & survival[8]);

        Leaf(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blinker() {
        let life = Rule::new(&[3], &[2, 3]);
        let blinker = Leaf::from_array([
            0b_00000000,
            0b_00000000,
            0b_00000000,
            0b_00011100,
            0b_00000000,
            0b_00000000,
            0b_00000000,
            0b_00000000,
        ]);
        assert_eq!(blinker.tick(&life).tick(&life), blinker);
    }
}
