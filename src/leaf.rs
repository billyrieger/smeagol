// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

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
    /// # use smeagol::leaf::Rule;
    /// // Conway's Game of Life: B3/S23
    /// let life = Rule::new(&[3], &[2, 3]);
    /// ```
    pub fn new(birth: &[usize], survival: &[usize]) -> Self {
        Self {
            birth: make_rule(birth),
            survival: make_rule(survival),
        }
    }
}

fn make_rule(neighbors: &[usize]) -> [Bool8x8; 9] {
    let mut result = [Bool8x8::FALSE; 9];
    for &i in neighbors {
        result.get_mut(i).map(|x| *x = Bool8x8::TRUE);
    }
    result
}

/// An 8 by 8 grid of dead or alive cells.
///
/// ```
/// # use smeagol::leaf::{Bool8x8, Leaf};
/// let glider = Leaf::new(Bool8x8(0x0000_1008_3800_0000));
/// ```
///
/// ```text
/// ┏━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┓                  
/// ┃ ░░   ░░   ░░   ░░ ╎ ░░   ░░   ░░   ░░ ┃ 0x00 = 0000 0000   
/// ┠                   ╎                   ┨                   
/// ┃ ░░   ░░   ░░   ░░ ╎ ░░   ░░   ░░   ░░ ┃ 0x00 = 0000 0000   
/// ┠                   ╎                   ┨                   
/// ┃ ░░   ░░   ░░   ██ ╎ ░░   ░░   ░░   ░░ ┃ 0x10 = 0001 0000   
/// ┠                   ╎                   ┨                   
/// ┃ ░░   ░░   ░░   ░░ ╎ ██   ░░   ░░   ░░ ┃ 0x08 = 0000 1000   
/// ┠  ╌  ╌ ╌  ╌ ╌  ╌ ╌   ╌ ╌  ╌ ╌  ╌ ╌  ╌  ┨                   
/// ┃ ░░   ░░   ██   ██ ╎ ██   ░░   ░░   ░░ ┃ 0x38 = 0011 1000   
/// ┠                   ╎                   ┨                   
/// ┃ ░░   ░░   ░░   ░░ ╎ ░░   ░░   ░░   ░░ ┃ 0x00 = 0000 0000   
/// ┠                   ╎                   ┨                   
/// ┃ ░░   ░░   ░░   ░░ ╎ ░░   ░░   ░░   ░░ ┃ 0x00 = 0000 0000   
/// ┠                   ╎                   ┨                   
/// ┃ ░░   ░░   ░░   ░░ ╎ ░░   ░░   ░░   ░░ ┃ 0x00 = 0000 0000   
/// ┗━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┛                  
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Leaf {
    pub alive: Bool8x8,
}

impl Leaf {
    /// # Examples
    ///
    /// ```
    /// # use smeagol::leaf::{Bool8x8, Leaf};
    /// let glider = Leaf::new(Bool8x8(0x0000_1008_3800_0000));
    /// ```
    pub fn new(alive: Bool8x8) -> Self {
        Self { alive }
    }

    pub fn mask(self, mask: Bool8x8) -> Self {
        Self::new(self.alive & mask)
    }

    pub fn population(&self) -> u128 {
        u128::from(self.alive.0.count_ones())
    }

    pub fn step(&self, rule: Rule) -> Self {
        let (alive, dead) = (self.alive, !self.alive);

        let alive_neighbor_count = Bool8x8::sum(&[
            alive.up(1),
            alive.down(1),
            alive.left(1),
            alive.right(1),
            alive.up(1).left(1),
            alive.up(1).right(1),
            alive.down(1).left(1),
            alive.left(1).right(1),
        ]);

        let alive_next = Bool8x8::FALSE
            | dead & alive_neighbor_count[0] & rule.birth[0]
            | dead & alive_neighbor_count[1] & rule.birth[1]
            | dead & alive_neighbor_count[2] & rule.birth[2]
            | dead & alive_neighbor_count[3] & rule.birth[3]
            | dead & alive_neighbor_count[4] & rule.birth[4]
            | dead & alive_neighbor_count[5] & rule.birth[5]
            | dead & alive_neighbor_count[6] & rule.birth[6]
            | dead & alive_neighbor_count[7] & rule.birth[7]
            | dead & alive_neighbor_count[8] & rule.birth[8]
            | alive & alive_neighbor_count[0] & rule.survival[0]
            | alive & alive_neighbor_count[1] & rule.survival[1]
            | alive & alive_neighbor_count[2] & rule.survival[2]
            | alive & alive_neighbor_count[3] & rule.survival[3]
            | alive & alive_neighbor_count[4] & rule.survival[4]
            | alive & alive_neighbor_count[5] & rule.survival[5]
            | alive & alive_neighbor_count[6] & rule.survival[6]
            | alive & alive_neighbor_count[7] & rule.survival[7]
            | alive & alive_neighbor_count[8] & rule.survival[8];

        Self::new(alive_next)
    }

    pub fn jump(&self, rule: Rule) -> Self {
        self.step(rule).step(rule)
    }
}

/// A `u64` interpreted as an 8 by 8 grid of booleans.
///
/// The following diagram shows the layout of the bits of a `u64` to make a
/// square. The most significant bit, `1 << 63`, is in the upper-left corner
/// and the least significant bit, `1 << 0`, is in the bottom-right.
///
/// ```text
/// ┌────┬────┬────┬────┬────┬────┬────┬────┐
/// │ 63 │ 62 │ 61 │ 60 │ 59 │ 58 │ 57 │ 56 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │ 55 │ 54 │ 53 │ 52 │ 51 │ 50 │ 49 │ 48 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │ 47 │ 46 │ 45 │ 44 │ 43 │ 42 │ 41 │ 40 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │ 39 │ 38 │ 37 │ 36 │ 35 │ 34 │ 33 │ 32 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │ 31 │ 30 │ 29 │ 28 │ 27 │ 26 │ 25 │ 24 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │ 23 │ 22 │ 21 │ 20 │ 19 │ 18 │ 17 │ 16 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │ 15 │ 14 │ 13 │ 12 │ 11 │ 10 │  9 │  8 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │  7 │  6 │  5 │  4 │  3 │  2 │  1 │  0 │
/// └────┴────┴────┴────┴────┴────┴────┴────┘
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bool8x8(pub u64);

impl Bool8x8 {
    /// The `Bool8x8` where all elements are `false`.
    pub const FALSE: Self = Self(0);

    /// The `Bool8x8` where all elements are `true`.
    pub const TRUE: Self = Self(u64::MAX);

    /// Shifts the `Bool8x8` to the left by the given number of steps.
    pub fn left(&self, steps: u8) -> Self {
        Self(self.0 << steps)
    }

    /// Shifts the `Bool8x8` to the right by the given number of steps.
    pub fn right(&self, steps: u8) -> Self {
        Self(self.0 >> steps)
    }

    /// Shifts the `Bool8x8` up by the given number of steps.
    pub fn up(&self, steps: u8) -> Self {
        Self(self.0 << (steps * 8))
    }

    /// Shifts the `Bool8x8` down by the given number of steps.
    pub fn down(&self, steps: u8) -> Self {
        Self(self.0 >> (steps * 8))
    }

    pub fn sum(addends: &[Self]) -> [Self; 9] {
        let mut digits = [Self::FALSE; 4];

        for &addend in addends {
            // add `addend` to the first digit `digits[0]`
            let carry0 = digits[0] & addend;
            digits[0] ^= addend;

            // add `carry0` to the next digit `digits[1]`
            let carry1 = digits[1] & carry0;
            digits[1] ^= carry0;

            // add `carry1` to the next digit `digits[2]`
            let carry2 = digits[2] & carry1;
            digits[2] ^= carry1;

            // add `carry2` to the final digit `digits[3]`
            digits[3] |= carry2;
        }

        let [a1, b1, c1, d1] = digits;
        let [a0, b0, c0, d0] = [!a1, !b1, !c1, !d1];
        [
            d0 & c0 & b0 & a0, // 0000 = 0
            d0 & c0 & b0 & a1, // 0001 = 1
            d0 & c0 & b1 & a0, // 0010 = 2
            d0 & c0 & b1 & a1, // 0011 = 3
            d0 & c1 & b0 & a0, // 0100 = 4
            d0 & c1 & b0 & a1, // 0101 = 5
            d0 & c1 & b1 & a0, // 0110 = 6
            d0 & c1 & b1 & a1, // 0111 = 7
            d1 & c0 & b0 & a0, // 1000 = 8
        ]
    }
}

impl BitAnd for Bool8x8 {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl BitAndAssign for Bool8x8 {
    fn bitand_assign(&mut self, other: Self) {
        *self = *self & other;
    }
}

impl BitOr for Bool8x8 {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitOrAssign for Bool8x8 {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

impl BitXor for Bool8x8 {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }
}

impl BitXorAssign for Bool8x8 {
    fn bitxor_assign(&mut self, other: Self) {
        *self = *self ^ other;
    }
}

impl Not for Bool8x8 {
    type Output = Self;

    fn not(self) -> Self {
        Self(!self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adder_histogram() {
        let buckets = Bool8x8::sum(&[
            Bool8x8(0x00000000F0000000),
            Bool8x8(0x0000000FFF000000),
            Bool8x8(0x000000FFFFF00000),
            Bool8x8(0x00000FFFFFFF0000),
            Bool8x8(0x0000FFFFFFFFF000),
            Bool8x8(0x000FFFFFFFFFFF00),
            Bool8x8(0x00FFFFFFFFFFFFF0),
            Bool8x8(0x0FFFFFFFFFFFFFFF),
        ]);

        assert_eq!(Bool8x8(0x00000000F0000000), buckets[8]);
        assert_eq!(Bool8x8(0x0000000F0F000000), buckets[7]);
        assert_eq!(Bool8x8(0x000000F000F00000), buckets[6]);
        assert_eq!(Bool8x8(0x00000F00000F0000), buckets[5]);
        assert_eq!(Bool8x8(0x0000F0000000F000), buckets[4]);
        assert_eq!(Bool8x8(0x000F000000000F00), buckets[3]);
        assert_eq!(Bool8x8(0x00F00000000000F0), buckets[2]);
        assert_eq!(Bool8x8(0x0F0000000000000F), buckets[1]);
        assert_eq!(Bool8x8(0xF000000000000000), buckets[0]);
    }

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
