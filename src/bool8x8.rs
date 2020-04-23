// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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
///
/// ┏━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┓
/// ┃ 63   62   61   60 ╎ 59   58   57   56 ┃
/// ┠                   ╎                   ┨
/// ┃ 55   54   53   52 ╎ 51   50   49   48 ┃
/// ┠                   ╎                   ┨
/// ┃ 47   46   45   44 ╎ 43   42   41   40 ┃
/// ┠                   ╎                   ┨
/// ┃ 39   38   37   36 ╎ 35   34   33   32 ┃
/// ┠ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌   ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ┨
/// ┃ 31   30   29   28 ╎ 27   26   25   24 ┃
/// ┠                   ╎                   ┨
/// ┃ 23   22   21   20 ╎ 19   18   17   16 ┃
/// ┠                   ╎                   ┨
/// ┃ 15   14   13   12 ╎ 11   10    9    8 ┃
/// ┠                   ╎                   ┨
/// ┃  7    6    5    4 ╎  3    2    1    0 ┃
/// ┗━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┛
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bool8x8(pub u64);

impl Bool8x8 {
    /// The `Bool8x8` where all elements are `false`.
    pub const FALSE: Self = Self(0);

    /// The `Bool8x8` where all elements are `true`.
    pub const TRUE: Self = Self(u64::MAX);

    /// Performs an element-wise boolean AND operation.
    pub const fn and(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }

    /// Performs an element-wise boolean OR operation.
    pub const fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Performs an element-wise boolean XOR operation.
    pub const fn xor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }

    /// Performs an element-wise boolean NOT operation.
    pub const fn not(self) -> Self {
        Self(!self.0)
    }

    /// Shifts the `Bool8x8` to the left by the given number of steps.
    pub const fn left(&self, steps: u8) -> Self {
        Self(self.0 << steps)
    }

    /// Shifts the `Bool8x8` to the right by the given number of steps.
    pub const fn right(&self, steps: u8) -> Self {
        Self(self.0 >> steps)
    }

    /// Shifts the `Bool8x8` up by the given number of steps.
    pub const fn up(&self, steps: u8) -> Self {
        Self(self.0 << (steps * 8))
    }

    /// Shifts the `Bool8x8` down by the given number of steps.
    pub const fn down(&self, steps: u8) -> Self {
        Self(self.0 >> (steps * 8))
    }

    const fn half_adder(a: Self, b: Self) -> (Self, Self) {
        (a.xor(b), a.and(b))
    }
}

pub struct Adder {
    digits: [Bool8x8; 4],
}

impl Adder {
    /// Creates a new empty `Adder`.
    pub const fn new() -> Self {
        Self {
            digits: [Bool8x8::FALSE; 4],
        }
    }

    pub const fn add(self, input: Bool8x8) -> Self {
        let [a, b, c, d] = self.digits;

        // add the first digit to the input
        let (w, carry) = Bool8x8::half_adder(a, input);

        // add the next digit to the previous carry
        let (x, carry) = Bool8x8::half_adder(b, carry);

        // add the next digit to the previous carry
        let (y, carry) = Bool8x8::half_adder(c, carry);

        // saturating add the final digit to the previous carry
        let z = d.or(carry);

        Self {
            digits: [w, x, y, z],
        }
    }

    pub const fn sum(self) -> [Bool8x8; 9] {
        let [a1, b1, c1, d1] = self.digits;
        let [a0, b0, c0, d0] = [a1.not(), b1.not(), c1.not(), d1.not()];
        [
            d0.and(c0).and(b0).and(a0), // 0000 = 0
            d0.and(c0).and(b0).and(a1), // 0001 = 1
            d0.and(c0).and(b1).and(a0), // 0010 = 2
            d0.and(c0).and(b1).and(a1), // 0011 = 3
            d0.and(c1).and(b0).and(a0), // 0100 = 4
            d0.and(c1).and(b0).and(a1), // 0101 = 5
            d0.and(c1).and(b1).and(a0), // 0110 = 6
            d0.and(c1).and(b1).and(a1), // 0111 = 7
            d1.and(c0).and(b0).and(a0), // 1000 = 8
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram() {
        let buckets = Adder::new()
            .add(Bool8x8(0x0000000F00000000))
            .add(Bool8x8(0x000000FFF0000000))
            .add(Bool8x8(0x00000FFFFF000000))
            .add(Bool8x8(0x0000FFFFFFF00000))
            .add(Bool8x8(0x000FFFFFFFFF0000))
            .add(Bool8x8(0x00FFFFFFFFFFF000))
            .add(Bool8x8(0x0FFFFFFFFFFFFF00))
            .add(Bool8x8(0xFFFFFFFFFFFFFFF0))
            .sum();

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
}
