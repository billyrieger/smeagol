// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::core::Offset;

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

/// A `u64` interpreted as a square grid of boolean values.
///
/// # Bit layout
///
/// The following diagram shows the layout of the bits of a `u64` to make a
/// square grid. The most significant bit, `1 << 63`, is in the upper-left
/// corner and the least significant bit, `1 << 0`, is in the bottom-right. Each
/// row of the grid corresponds to one contiguous byte of the `u64`.
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

pub type SumResult = [Bool8x8; 9];

impl Bool8x8 {
    pub const FALSE: Self = Self(0);
    pub const TRUE: Self = Self(!0);

    pub const WEST: Self = Self(0x_F0_F0_F0_F0_F0_F0_F0_F0);
    pub const EAST: Self = Self(0x_0F_0F_0F_0F_0F_0F_0F_0F);

    pub const NORTH: Self = Self(0x_FF_FF_FF_FF_00_00_00_00);
    pub const SOUTH: Self = Self(0x_00_00_00_00_FF_FF_FF_FF);

    pub const NORTHWEST: Self = Self(0x_F0_F0_F0_F0_00_00_00_00);
    pub const NORTHEAST: Self = Self(0x_0F_0F_0F_0F_00_00_00_00);
    pub const SOUTHWEST: Self = Self(0x_00_00_00_00_F0_F0_F0_F0);
    pub const SOUTHEAST: Self = Self(0x_00_00_00_00_0F_0F_0F_0F);

    pub const CENTER: Self = Self(0x_00_00_3C_3C_3C_3C_00_00);

    pub fn get_bit(&self, index: usize) -> bool {
        self.0 & (1 << index) > 0
    }

    pub fn set_bit(&self, index: usize) -> Self {
        Self(self.0 | (1 << index))
    }

    pub fn unset_bit(&self, index: usize) -> Self {
        Self(self.0 & !(1 << index))
    }

    pub fn shift(&self, offset: Offset) -> Self {
        let shift = match offset {
            Offset::West(dx) => -dx,
            Offset::East(dx) => dx,
            Offset::North(dy) => -8 * dy,
            Offset::South(dy) => 8 * dy,
            Offset::Northwest(delta) => -9 * delta,
            Offset::Northeast(delta) => -7 * delta,
            Offset::Southwest(delta) => 7 * delta,
            Offset::Southeast(delta) => 9 * delta,
            Offset::Arbitrary { dx, dy } => 8 * dy + dx,
        };
        if shift >= 0 {
            Self(self.0 >> shift)
        } else {
            Self(self.0 << -shift)
        }
    }

    pub fn sum(addends: &[Bool8x8]) -> [Bool8x8; 9] {
        // fun fact: the first term in a sum is called the `augend`
        let half_adder = |augend: &mut Bool8x8, addend: Bool8x8| -> Bool8x8 {
            let carry = *augend & addend;
            *augend ^= addend;
            carry
        };

        let mut digits = [Bool8x8::FALSE; 4];

        for &addend in addends {
            let carry0 = half_adder(&mut digits[3], addend);
            let carry1 = half_adder(&mut digits[2], carry0);
            let carry2 = half_adder(&mut digits[1], carry1);
            digits[0] |= carry2;
        }

        let [a1, b1, c1, d1] = digits;
        let [a0, b0, c0, d0] = [!a1, !b1, !c1, !d1];

        [
            a0 & b0 & c0 & d0, // 0000 = 0
            a0 & b0 & c0 & d1, // 0001 = 1
            a0 & b0 & c1 & d0, // 0010 = 2
            a0 & b0 & c1 & d1, // 0011 = 3
            a0 & b1 & c0 & d0, // 0100 = 4
            a0 & b1 & c0 & d1, // 0101 = 5
            a0 & b1 & c1 & d0, // 0110 = 6
            a0 & b1 & c1 & d1, // 0111 = 7
            a1 & b0 & c0 & d0, // 1000 = 8
        ]
    }
}

impl BitAnd for Bool8x8 {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        Bool8x8(self.0 & other.0)
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
        Bool8x8(self.0 | other.0)
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
        Bool8x8(self.0 ^ other.0)
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

    type B = Bool8x8;

    #[test]
    fn shift() {
        assert_eq!(B::CENTER.shift(Offset::Northwest(2)), B::NORTHWEST);
        assert_eq!(B::CENTER.shift(Offset::Northeast(2)), B::NORTHEAST);
        assert_eq!(B::CENTER.shift(Offset::Southwest(2)), B::SOUTHWEST);
        assert_eq!(B::CENTER.shift(Offset::Southeast(2)), B::SOUTHEAST);
    }

    #[test]
    fn adder() {
        let buckets = Bool8x8::sum(&[
            Bool8x8(0x0000000F00000000),
            Bool8x8(0x000000FFF0000000),
            Bool8x8(0x00000FFFFF000000),
            Bool8x8(0x0000FFFFFFF00000),
            Bool8x8(0x000FFFFFFFFF0000),
            Bool8x8(0x00FFFFFFFFFFF000),
            Bool8x8(0x0FFFFFFFFFFFFF00),
            Bool8x8(0xFFFFFFFFFFFFFFF0),
        ]);

        assert_eq!(Bool8x8(0x0000000F00000000), buckets[8]);
        assert_eq!(Bool8x8(0x000000F0F0000000), buckets[7]);
        assert_eq!(Bool8x8(0x00000F000F000000), buckets[6]);
        assert_eq!(Bool8x8(0x0000F00000F00000), buckets[5]);
        assert_eq!(Bool8x8(0x000F0000000F0000), buckets[4]);
        assert_eq!(Bool8x8(0x00F000000000F000), buckets[3]);
        assert_eq!(Bool8x8(0x0F00000000000F00), buckets[2]);
        assert_eq!(Bool8x8(0xF0000000000000F0), buckets[1]);
        assert_eq!(Bool8x8(0x000000000000000F), buckets[0]);
    }

    #[test]
    fn partitions() {
        // returns true iff the union of all the masks is the entire grid
        let covers_grid = |masks: &[B]| masks.iter().copied().fold(B::FALSE, B::bitor) == B::TRUE;

        // returns true iff the intersection of any two masks is empty
        let pairwise_disjoint = |masks: &[B]| {
            let len = masks.len();
            for i in 0..len {
                for j in (i + 1)..len {
                    if masks[i] & masks[j] != B::FALSE {
                        return false;
                    }
                }
            }
            true
        };

        let partitions: &[&[_]] = &[
            &[B::TRUE],
            &[B::NORTH, B::SOUTH],
            &[B::EAST, B::WEST],
            &[B::NORTHWEST, B::NORTHEAST, B::SOUTHWEST, B::SOUTHEAST],
            &[B::NORTH, B::SOUTHWEST, B::SOUTHEAST],
            &[B::FALSE, B::FALSE, B::FALSE, B::TRUE, B::FALSE],
        ];

        for partition in partitions {
            assert!(covers_grid(partition));
            assert!(pairwise_disjoint(partition));
        }
    }
}
