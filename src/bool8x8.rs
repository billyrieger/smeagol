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
    pub const TRUE: Self = Self(u64::MAX);

    pub const NORTH: Self = Self(0xFFFF_FFFF_0000_0000);
    pub const SOUTH: Self = Self(0x0000_0000_FFFF_FFFF);
    pub const EAST: Self = Self(0x00FF_00FF_00FF_00FF);
    pub const WEST: Self = Self(0xFF00_FF00_FF00_FF00);

    pub const NORTHWEST: Self = Self(0xF0F0_F0F0_0000_0000);
    pub const NORTHEAST: Self = Self(0x0F0F_0F0F_0000_0000);
    pub const SOUTHWEST: Self = Self(0x0000_0000_F0F0_F0F0);
    pub const SOUTHEAST: Self = Self(0x0000_0000_0F0F_0F0F);
    pub const CENTER: Self = Self(0x0000_3C3C_3C3C_0000);

    pub const fn and(&self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub const fn or(&self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn xor(&self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }

    pub const fn fold_and(result: Self, list: &[Self]) -> Self {
        match list {
            [] => result,
            &[head, ref tail @ ..] => Self::fold_and(result.and(head), tail),
        }
    }

    pub const fn fold_or(result: Self, list: &[Self]) -> Self {
        match list {
            &[] => result,
            &[head, ref tail @ ..] => Self::fold_or(result.or(head), tail),
        }
    }

    pub const fn fold_xor(result: Self, list: &[Self]) -> Self {
        match list {
            [] => result,
            &[head, ref tail @ ..] => Self::fold_xor(result.xor(head), tail),
        }
    }

    const fn any_both_helper(result: Self, xs: &[Self], ys: &[Self]) -> Self {
        match (xs, ys) {
            (&[], &[]) => result,
            (&[], &[..]) => Self::FALSE,
            (&[..], &[]) => Self::FALSE,
            (&[x, ref xs @ ..], &[y, ref ys @ ..]) => {
                Self::any_both_helper(result.or(x.and(y)), xs, ys)
            }
        }
    }

    pub const fn all(list: &[Self]) -> Self {
        Self::fold_and(Self::TRUE, list)
    }

    pub const fn any(list: &[Self]) -> Self {
        Self::fold_or(Self::FALSE, list)
    }

    pub const fn any_both(xs: &[Self], ys: &[Self]) -> Self {
        Self::any_both_helper(Self::FALSE, xs, ys)
    }

    /// Performs bitwise boolean NOT.
    pub const fn not(&self) -> Self {
        Self(!self.0)
    }

    pub const fn offset(&self, dx: i8, dy: i8) -> Self {
        let mut result = self.0;

        if dx >= 0 {
            result >>= dx;
        } else {
            result <<= -dx;
        }

        if dy >= 0 {
            result <<= 8 * dy;
        } else {
            result >>= -8 * dy
        };

        Self(result)
    }

    /// Performs a bitwise sum of the given `Bool8x8`s.
    ///
    /// The adder can only count to 8.
    pub const fn sum(addends: &[Self]) -> SumResult {
        let [a1, b1, c1, d1] = Self::adder([Self::FALSE; 4], addends);
        let [a0, b0, c0, d0] = [a1.not(), b1.not(), c1.not(), d1.not()];
        [
            Self::all(&[a0, b0, c0, d0]), // 0000 = 0
            Self::all(&[a0, b0, c0, d1]), // 0001 = 1
            Self::all(&[a0, b0, c1, d0]), // 0010 = 2
            Self::all(&[a0, b0, c1, d1]), // 0011 = 3
            Self::all(&[a0, b1, c0, d0]), // 0100 = 4
            Self::all(&[a0, b1, c0, d1]), // 0101 = 5
            Self::all(&[a0, b1, c1, d0]), // 0110 = 6
            Self::all(&[a0, b1, c1, d1]), // 0111 = 7
            Self::all(&[a1, b0, c0, d0]), // 1000 = 8
        ]
    }

    const fn adder(sum: [Self; 4], addends: &[Self]) -> [Self; 4] {
        // digits of sum in big-endian binary encoding
        let [a, b, c, d] = sum;
        match addends {
            [] => sum,
            &[addend, ref tail @ ..] => {
                let (d, carry) = (d.xor(addend), d.and(addend));
                let (c, carry) = (c.xor(carry), c.and(carry));
                let (b, carry) = (b.xor(carry), b.and(carry));
                let a = a.or(carry);
                Self::adder([a, b, c, d], tail)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partitions() {
        type B = Bool8x8;

        let check = |masks: &[B]| B::fold_xor(B::FALSE, masks) == B::TRUE;

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
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x18 | . . . # # . . .
        // 0x18 | . . . # # . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let center = Bool8x8(0x0000_0018_1800_0000);

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x18 | . . . # # . . .
        // 0x18 | . . . # # . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let north = Bool8x8(0x0000_1818_0000_0000);

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x18 | . . . # # . . .
        // 0x18 | . . . # # . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let south = Bool8x8(0x0000_0000_1818_0000);

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x0C | . . . . # # . .
        // 0x0C | . . . . # # . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let east = Bool8x8(0x0000_000C_0C00_0000);

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x30 | . . # # . . . .
        // 0x30 | . . # # . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let west = Bool8x8(0x0000_0030_3000_0000);

        assert_eq!(center.offset(1, 0), east);
        assert_eq!(center.offset(-1, 0), west);
        assert_eq!(center.offset(0, 1), north);
        assert_eq!(center.offset(0, -1), south);
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
