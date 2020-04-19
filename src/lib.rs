// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(const_fn, const_if_match)]

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
///
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Leaf {
    cells: u64,
}

impl Leaf {
    /// Creates a `Leaf` from a grid of cells represented by a `u64`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use smeagol::Leaf;
    /// // . . . . . . . .
    /// // . . . . . . . .
    /// // . . . . @ . . .
    /// // . . . . . @ . .
    /// // . . . @ @ @ . .
    /// // . . . . . . . .
    /// // . . . . . . . .
    /// // . . . . . . . .
    /// let glider = Leaf::new(
    ///     0b_00000000_00000000_00001000_00000100_00011100_00000000_00000000_00000000
    /// );
    /// ```
    pub const fn new(cells: u64) -> Self {
        Self { cells }
    }

    pub const fn set(&self, index: u8) -> Self {
        Self::new(self.cells | 1 << index)
    }

    pub const fn unset(&self, index: u8) -> Self {
        Self::new(self.cells & !(1 << index))
    }

    const fn right(&self, steps: u8) -> Self {
        Self::new(self.cells >> steps)
    }

    const fn left(&self, steps: u8) -> Self {
        Self::new(self.cells << steps)
    }

    const fn down(&self, steps: u8) -> Self {
        Self::new(self.cells >> (steps * 8))
    }

    const fn up(&self, steps: u8) -> Self {
        Self::new(self.cells << (steps * 8))
    }
}

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
            .add(leaf.up(1).cells)
            .add(leaf.down(1).cells)
            .add(leaf.left(1).cells)
            .add(leaf.right(1).cells)
            .add(leaf.up(1).left(1).cells)
            .add(leaf.up(1).right(1).cells)
            .add(leaf.down(1).left(1).cells)
            .add(leaf.down(1).right(1).cells)
            .finish();

        let alive = leaf.cells;
        let dead = !leaf.cells;
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

        Leaf::new(result)
    }
}

struct Counter {
    bits: [u64; 4],
}

impl Counter {
    /// Returns an empty counter initialized to 0.
    const fn new() -> Self {
        Self { bits: [0; 4] }
    }

    const fn add(self, i: u64) -> Self {
        let [a, b, c, d] = self.bits;

        let b0 = a ^ i; // sum
        let q = a & i; // carry

        let b1 = b ^ q; // sum
        let q = b & q; // carry

        let b2 = c ^ q; // sum
        let q = c & q; // carry

        let b3 = d | q; // saturating sum

        Self {
            bits: [b0, b1, b2, b3],
        }
    }

    #[rustfmt::skip]
    const fn finish(self) -> [u64; 9] {
        let [q0, q1, q2, q3] = self.bits;
        [
            !q3 & !q2 & !q1 & !q0, // 0000 = 0
            !q3 & !q2 & !q1 &  q0, // 0001 = 1
            !q3 & !q2 &  q1 & !q0, // 0010 = 2
            !q3 & !q2 &  q1 &  q0, // 0011 = 3
            !q3 &  q2 & !q1 & !q0, // 0100 = 4
            !q3 &  q2 & !q1 &  q0, // 0101 = 5
            !q3 &  q2 &  q1 & !q0, // 0110 = 6
            !q3 &  q2 &  q1 &  q0, // 0111 = 7
             q3 & !q2 & !q1 & !q0, // 1000 = 8
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blinker() {
        let conway = Rule::new(&[3], &[2, 3]);
        let blinker = Leaf::new(0b_00000000_00011100_00000000_00000000_00000000_00000000);
        let flipped = Leaf::new(0b_00001000_00001000_00001000_00000000_00000000_00000000);
        assert_eq!(conway.step(blinker), flipped);
        assert_eq!(conway.step(flipped), blinker);
    }

    #[test]
    fn counter_histogram() {
        let sums = Counter::new()
            //      |1234567876543210|
            .add(0x__0000000F00000000)
            .add(0x__000000FFF0000000)
            .add(0x__00000FFFFF000000)
            .add(0x__0000FFFFFFF00000)
            .add(0x__000FFFFFFFFF0000)
            .add(0x__00FFFFFFFFFFF000)
            .add(0x__0FFFFFFFFFFFFF00)
            .add(0x__FFFFFFFFFFFFFFF0)
            //      |1234567876543210|
            .finish();
        //          |1234567876543210|
        assert_eq!(0x0000000F00000000, sums[8]);
        assert_eq!(0x000000F0F0000000, sums[7]);
        assert_eq!(0x00000F000F000000, sums[6]);
        assert_eq!(0x0000F00000F00000, sums[5]);
        assert_eq!(0x000F0000000F0000, sums[4]);
        assert_eq!(0x00F000000000F000, sums[3]);
        assert_eq!(0x0F00000000000F00, sums[2]);
        assert_eq!(0xF0000000000000F0, sums[1]);
        assert_eq!(0x000000000000000F, sums[0]);
        //          |1234567876543210|
    }
}
