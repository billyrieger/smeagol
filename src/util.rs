// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    hash::Hash,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Offset {
    West(u8),
    East(u8),
    North(u8),
    South(u8),
    Northwest(u8),
    Northeast(u8),
    Southwest(u8),
    Southeast(u8),
}

impl Bool8x8 {
    pub const FALSE: Self = Self(u64::MIN);
    pub const TRUE: Self = Self(u64::MAX);

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

    pub fn set_bit(&self, index: usize, value: bool) -> Self {
        if value {
            Self(self.0 | (1 << index))
        } else {
            Self(self.0 & !(1 << index))
        }
    }

    pub fn offset(&self, offset: Offset) -> Self {
        match offset {
            Offset::West(dx) => Self(self.0 << dx),
            Offset::East(dx) => Self(self.0 >> dx),
            Offset::North(dy) => Self(self.0 << 8 * dy),
            Offset::South(dy) => Self(self.0 >> 8 * dy),
            Offset::Northwest(delta) => Self(self.0 << 9 * delta),
            Offset::Northeast(delta) => Self(self.0 << 7 * delta),
            Offset::Southwest(delta) => Self(self.0 >> 7 * delta),
            Offset::Southeast(delta) => Self(self.0 >> 9 * delta),
        }
    }

    pub fn sum(addends: &[Bool8x8]) -> SumResult {
        let half_adder = |sum: &mut Bool8x8, addend: Bool8x8| -> Bool8x8 {
            let carry = *sum & addend;
            *sum ^= addend;
            carry
        };

        let mut digits = [Bool8x8::FALSE; 4];

        for &addend in addends {
            let carry = half_adder(&mut digits[3], addend);
            let carry = half_adder(&mut digits[2], carry);
            let carry = half_adder(&mut digits[1], carry);
            digits[0] |= carry;
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid2<T>(pub [T; 4]);

impl<T> Grid2<T>
where
    T: Copy,
{
    pub fn map<F, U>(&self, mut f: F) -> Grid2<U>
    where
        F: FnMut(T) -> U,
    {
        let [a, b, c, d] = self.0;
        Grid2([f(a), f(b), f(c), f(d)])
    }

    pub fn try_map<E, F, U>(&self, mut f: F) -> Result<Grid2<U>, E>
    where
        F: FnMut(T) -> Result<U, E>,
    {
        let [a, b, c, d] = self.0;
        Ok(Grid2([f(a)?, f(b)?, f(c)?, f(d)?]))
    }
}

impl<T> Grid2<Grid2<T>>
where
    T: Copy,
{
    pub fn flatten(&self) -> Grid4<T> {
        // a b | c d
        // e f | g h
        // ----+----
        // i j | k l
        // m n | o p
        let [[a, b, e, f], [c, d, g, h], [i, j, m, n], [k, l, o, p]] = self.map(|grid| grid.0).0;
        Grid4([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p])
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid3<T>(pub [T; 9]);

impl<T> Grid3<T>
where
    T: Copy,
{
    pub fn shrink<E, F, U>(&self, mut map: F) -> Result<Grid2<U>, E>
    where
        F: FnMut(Grid2<T>) -> Result<U, E>,
    {
        // a---b---c
        // | w | x |
        // d---e---f
        // | y | z |
        // g---h---i
        let [a, b, c, d, e, f, g, h, i] = self.0;
        let w = map(Grid2([a, b, d, e]))?;
        let x = map(Grid2([b, c, e, f]))?;
        let y = map(Grid2([d, e, g, h]))?;
        let z = map(Grid2([e, f, h, i]))?;
        Ok(Grid2([w, x, y, z]))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid4<T>(pub [T; 16]);

impl<T> Grid4<T>
where
    T: Copy,
{
    pub fn center(&self) -> Grid2<T> {
        let [_, _, _, _, _, f, g, _, _, j, k, _, _, _, _, _] = self.0;
        Grid2([f, g, j, k])
    }

    pub fn shrink<E, F, U>(&self, mut func: F) -> Result<Grid3<U>, E>
    where
        F: FnMut(Grid2<T>) -> Result<U, E>,
    {
        // a---b---c---d
        // | r | s | t |
        // e---f---g---h
        // | u | v | w |
        // i---j---k---l
        // | x | y | z |
        // m---n---o---p
        let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = self.0;
        let r = func(Grid2([a, b, e, f]))?;
        let s = func(Grid2([b, c, f, g]))?;
        let t = func(Grid2([c, d, g, h]))?;
        let u = func(Grid2([e, f, i, j]))?;
        let v = func(Grid2([f, g, j, k]))?;
        let w = func(Grid2([g, h, k, l]))?;
        let x = func(Grid2([i, j, m, n]))?;
        let y = func(Grid2([j, k, n, o]))?;
        let z = func(Grid2([k, l, o, p]))?;
        Ok(Grid3([r, s, t, u, v, w, x, y, z]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    #[test]
    fn try_map() {
        let odd_cube = |x: u32| if x % 2 == 1 { Ok(x.pow(3)) } else { Err(()) };

        let odds = Grid2([1, 3, 5, 7]);
        assert_eq!(odds.try_map(odd_cube), Ok(Grid2([1, 27, 125, 343])));

        let not_odds = Grid2([1, 4, 3, 8]);
        assert_eq!(not_odds.try_map(odd_cube), Err(()));
    }

    #[test]
    fn flatten() {
        // 0 0 1 1
        // 0 0 1 1
        // 2 2 3 3
        // 2 2 3 3
        let zeros = Grid2([0; 4]);
        let ones = Grid2([1; 4]);
        let twos = Grid2([2; 4]);
        let threes = Grid2([3; 4]);
        let nested = Grid2([zeros, ones, twos, threes]);
        let flattened = nested.flatten();

        assert_eq!(
            flattened,
            Grid4([0, 0, 1, 1, 0, 0, 1, 1, 2, 2, 3, 3, 2, 2, 3, 3])
        );
    }

    #[test]
    fn shrink() {
        let sum = |grid: Grid2<u32>| -> Result<u32> { Ok(grid.0.iter().sum()) };
        let ones = Grid4([1; 16]);

        let fours = ones.shrink(sum).unwrap();
        assert_eq!(fours, Grid3([4; 9]));

        let sixteens = fours.shrink(sum).unwrap();
        assert_eq!(sixteens, Grid2([16; 4]));
    }

    #[test]
    fn partitions() {
        type B = Bool8x8;

        let check = |masks: &[B]| masks.iter().copied().fold(B::FALSE, B::bitxor) == Bool8x8::TRUE;

        let cases: &[&[_]] = &[
            &[B::TRUE],
            &[B::NORTH, B::SOUTH],
            &[B::EAST, B::WEST],
            &[B::NORTHWEST, B::NORTHEAST, B::SOUTHWEST, B::SOUTHEAST],
            &[B::FALSE, B::FALSE, B::TRUE, B::FALSE],
            &[B::SOUTHWEST, B::NORTH, B::FALSE, B::SOUTHEAST],
        ];

        for &case in cases {
            assert!(check(case));
        }
    }

    #[test]
    fn shift() {
        type B = Bool8x8;

        assert_eq!(B::CENTER.offset(Offset::Northwest(2)), B::NORTHWEST);
        assert_eq!(B::CENTER.offset(Offset::Northeast(2)), B::NORTHEAST);
        assert_eq!(B::CENTER.offset(Offset::Southwest(2)), B::SOUTHWEST);
        assert_eq!(B::CENTER.offset(Offset::Southeast(2)), B::SOUTHEAST);
    }

    #[test]
    fn adder() {
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
}
