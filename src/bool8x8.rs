// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// A `u64` interpreted as an 8 by 8 grid of boolean values.
///
/// ```txt
/// +---+---+---+---+---+---+---+---+
/// | 63| 62| 61| 60| 59| 58| 57| 56|
/// +---+---+---+---+---+---+---+---+
/// | 55| 54| 53| 52| 51| 50| 49| 48|
/// +---+---+---+---+---+---+---+---+
/// | 39| 38| 37| 36| 35| 34| 33| 32|
/// +---+---+---+---+---+---+---+---+
/// | 31| 30| 29| 28| 27| 26| 25| 24|
/// +---+---+---+---+---+---+---+---+
/// | 23| 22| 21| 20| 19| 18| 17| 16|
/// +---+---+---+---+---+---+---+---+
/// | 15| 14| 13| 12| 11| 10|  9|  8|
/// +---+---+---+---+---+---+---+---+
/// |  7|  6|  5|  4|  3|  2|  1|  0|
/// +---+---+---+---+---+---+---+---+
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bool8x8(u64);

impl Bool8x8 {
    pub const FALSE: Self = Self(0);
    pub const TRUE: Self = Self(u64::MAX);

    pub const fn from_bytes(bytes: [u8; 8]) -> Self {
        Self(u64::from_be_bytes(bytes))
    }

    /// Performs an element-wise boolean NOT operation.
    pub const fn not(self) -> Self {
        Self(!self.0)
    }

    /// Performs an element-wise boolean AND operation.
    pub const fn and(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Performs a boolean OR operation.
    pub const fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Performs a boolean XOR operation.
    pub const fn xor(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }

    pub const fn left(&self, steps: u8) -> Self {
        Self(self.0 << steps)
    }

    pub const fn right(&self, steps: u8) -> Self {
        Self(self.0 >> steps)
    }

    pub const fn up(&self, steps: u8) -> Self {
        Self(self.0 << (steps * 8))
    }

    pub const fn down(&self, steps: u8) -> Self {
        Self(self.0 >> (steps * 8))
    }
}

/// [half adder]
///
/// [half adder]: https://en.wikipedia.org/wiki/Adder_(electronics)#Half_adder
const fn half_adder(a: Bool8x8, b: Bool8x8) -> (Bool8x8, Bool8x8) {
    (a.xor(b), a.and(b))
}

pub struct Adder {
    digits: [Bool8x8; 4],
}

impl Adder {
    pub const fn new() -> Self {
        Self {
            digits: [Bool8x8::FALSE; 4],
        }
    }

    pub const fn add(self, input: Bool8x8) -> Self {
        let [b0, b1, b2, b3] = self.digits;

        // add the first digit to the input
        let (q0, carry) = half_adder(b0, input);
        // add the next digit to the previous carry
        let (q1, carry) = half_adder(b1, carry);
        // add the next digit to the previous carry
        let (q2, carry) = half_adder(b2, carry);
        // saturating add the final digit to the previous carry
        let q3 = b3.or(carry);

        Self {
            digits: [q0, q1, q2, q3],
        }
    }

    #[rustfmt::skip]
    pub const fn sum(self) -> [Bool8x8; 9] {
        let [b0, b1, b2, b3] = self.digits;
        let [nb0, nb1, nb2, nb3] = [b0.not(), b1.not(), b2.not(), b3.not()];

        // 0000
        let zero  = nb3.and(nb2).and(nb1).and(nb0); // 0000
        let one   = nb3.and(nb2).and(nb1).and( b0); // 0001
        let two   = nb3.and(nb2).and( b1).and(nb0); // 0010
        let three = nb3.and(nb2).and( b1).and( b0); // 0011
        let four  = nb3.and( b2).and(nb1).and(nb0); // 0100
        let five  = nb3.and( b2).and(nb1).and( b0); // 0101
        let six   = nb3.and( b2).and( b1).and(nb0); // 0110
        let seven = nb3.and( b2).and( b1).and( b0); // 0111
        let eight =  b3.and(nb2).and(nb1).and(nb0); // 1000

        [zero, one, two, three, four, five, six, seven, eight]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram() {
        let buckets = Adder::new()
            .add(Bool8x8(0x___0000000F00000000))
            .add(Bool8x8(0x___000000FFF0000000))
            .add(Bool8x8(0x___00000FFFFF000000))
            .add(Bool8x8(0x___0000FFFFFFF00000))
            .add(Bool8x8(0x___000FFFFFFFFF0000))
            .add(Bool8x8(0x___00FFFFFFFFFFF000))
            .add(Bool8x8(0x___0FFFFFFFFFFFFF00))
            .add(Bool8x8(0x___FFFFFFFFFFFFFFF0))
            .sum();

        assert_eq!(Bool8x8(0x_0000000F00000000), buckets[8]);
        assert_eq!(Bool8x8(0x_000000F0F0000000), buckets[7]);
        assert_eq!(Bool8x8(0x_00000F000F000000), buckets[6]);
        assert_eq!(Bool8x8(0x_0000F00000F00000), buckets[5]);
        assert_eq!(Bool8x8(0x_000F0000000F0000), buckets[4]);
        assert_eq!(Bool8x8(0x_00F000000000F000), buckets[3]);
        assert_eq!(Bool8x8(0x_0F00000000000F00), buckets[2]);
        assert_eq!(Bool8x8(0x_F0000000000000F0), buckets[1]);
        assert_eq!(Bool8x8(0x_000000000000000F), buckets[0]);
    }
}
