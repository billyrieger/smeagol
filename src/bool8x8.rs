use std::ops::{BitAnd, BitOr, BitXor, Not};

/// A `u64` interpreted as a square grid of boolean values.
///
/// # Bit layout
///
/// The following diagram shows the layout of the bits of a `u64` to make a
/// square grid. The most significant bit, `1 << 63`, is in the upper-left corner
/// and the least significant bit, `1 << 0`, is in the bottom-right. Each row of the grid
/// corresponds to one contiguous byte of the `u64`.
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
///
/// # Examples
///
/// ```
/// # use smeagol::Bool8x8;
/// // 00 | . . . . . . . .
/// // 3C | . . # # # # . .
/// // 20 | . . # . . . . .
/// // 38 | . . # # # . . .
/// // 20 | . . # . . . . .
/// // 20 | . . # . . . . .
/// // 20 | . . # . . . . .
/// // 00 | . . . . . . . .
/// let uppercase_f = Bool8x8(0x003C_2038_2020_2000);
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
        self.left(steps * 8)
    }

    /// Shifts the `Bool8x8` down by the given number of steps.
    pub fn down(&self, steps: u8) -> Self {
        self.right(steps * 8)
    }

    pub fn sum(addends: &[Self]) -> [Self; 9] {
        let mut digits = [Bool8x8::FALSE; 4];

        let half_adder = |a: &mut Self, b: Self| {
            *a = *a ^ b;
            (*a ^ b) & b
        };

        for &addend in addends {
            let carry = half_adder(&mut digits[0], addend);
            let carry = half_adder(&mut digits[1], carry);
            let carry = half_adder(&mut digits[2], carry);
            digits[3] = digits[3] | carry;
        }

        Self::finish_sum(digits)
    }

    // separate function to preserve formatting
    #[rustfmt::skip]
    fn finish_sum(digits: [Self; 4]) -> [Self; 9] {
        let [a, b, c, d] = digits;
        [
            !d & !c & !b & !a, // 0000 = 0
            !d & !c & !b &  a, // 0001 = 1
            !d & !c &  b & !a, // 0010 = 2
            !d & !c &  b &  a, // 0011 = 3
            !d &  c & !b & !a, // 0100 = 4
            !d &  c & !b &  a, // 0101 = 5
            !d &  c &  b & !a, // 0110 = 6
            !d &  c &  b &  a, // 0111 = 7
             d & !c & !b & !a, // 1000 = 8
        ]
    }
}

impl BitAnd for Bool8x8 {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl BitOr for Bool8x8 {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitXor for Bool8x8 {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
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
    fn shift() {
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 18 | . . . # # . . .
        // 18 | . . . # # . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        let center = Bool8x8(0x0000_0018_1800_0000);

        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 18 | . . . # # . . .
        // 18 | . . . # # . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        let north = Bool8x8(0x0000_1818_0000_0000);

        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 18 | . . . # # . . .
        // 18 | . . . # # . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        let south = Bool8x8(0x0000_0000_1818_0000);

        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 0C | . . . . # # . .
        // 0C | . . . . # # . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        let east = Bool8x8(0x0000_000C_0C00_0000);

        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 30 | . . # # . . . .
        // 30 | . . # # . . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        // 00 | . . . . . . . .
        let west = Bool8x8(0x0000_0030_3000_0000);

        assert_eq!(center.up(1), north);
        assert_eq!(center.down(1), south);
        assert_eq!(center.right(1), east);
        assert_eq!(center.left(1), west);
    }

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
}
