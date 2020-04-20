// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(const_fn, const_if_match)]

type Bool64 = u64;

mod consts {
    use super::*;

    pub const FALSE: Bool64 = 0x_00_00_00_00_00_00_00_00;
    pub const TRUE: Bool64 = 0x_FF_FF_FF_FF_FF_FF_FF_FF;
    pub const BORDER_TOP: Bool64 = 0x_FF_00_00_00_00_00_00_00;
    pub const BORDER_BOTTOM: Bool64 = 0x_00_00_00_00_00_00_00_FF;
    pub const BORDER_LEFT: Bool64 = 0x_80_80_80_80_80_80_80_80;
    pub const BORDER_RIGHT: Bool64 = 0x_01_01_01_01_01_01_01_01;

    pub const LEAF_CENTER: Leaf = Leaf(0x_00_00_3C_3C_3C_3C_00_00);
}

use consts::*;

/// An eight by eight grid of cells.
///
/// # Layout
///
/// ```txt
/// +---+---+---+---+---+---+---+---+
/// | 63| 62| 61| 60| 59| 58| 57| 56|
/// +---+---+---+---+---+---+---+---+
/// | 55| 54| 53| 52| 51| 50| 49| 48|
/// +---+---+---+---+---+---+---+---+
/// | 47| 46| 45| 44| 43| 42| 41| 40|
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Leaf(Bool64);

impl Leaf {
    pub const fn empty() -> Self {
        Self(FALSE)
    }

    pub const fn step(self, rule: Rule) -> Self {
        let birth = rule.birth;
        let survival = rule.survival;

        let sums = Adder::new().add_all(&self.neighbors()).sum();

        let is_alive = self.0;
        let is_dead = !self.0;
        let result = FALSE
            | sums[0] & (is_dead & birth[0] | is_alive & survival[0])
            | sums[1] & (is_dead & birth[1] | is_alive & survival[1])
            | sums[2] & (is_dead & birth[2] | is_alive & survival[2])
            | sums[3] & (is_dead & birth[3] | is_alive & survival[3])
            | sums[4] & (is_dead & birth[4] | is_alive & survival[4])
            | sums[5] & (is_dead & birth[5] | is_alive & survival[5])
            | sums[6] & (is_dead & birth[6] | is_alive & survival[6])
            | sums[7] & (is_dead & birth[7] | is_alive & survival[7])
            | sums[8] & (is_dead & birth[8] | is_alive & survival[8]);

        Leaf(result)
    }

    pub const fn step2(self, rule: Rule) -> Self {
        self.step(rule).step(rule).and(LEAF_CENTER)
    }

    pub const fn evolve(nw: Self, ne: Self, sw: Self, se: Self, rule: Rule) -> Self {
        // +-----------------+-----------------+
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . ~ ~ ~ ~ ~ ~ | ~ ~ . . . . . . |
        // | . . ~ ~ ~ ~ ~ ~ | ~ ~ . . . . . . |
        // | . . ~ ~ w w w w | ~ ~ . . . . . . |
        // | . . ~ ~ w w w w | ~ ~ . . . . . . |
        // | . . ~ ~ w w w w | ~ ~ . . . . . . |
        // | . . ~ ~ w w w w | ~ ~ . . . . . . |
        // | ----------------+---------------- |
        // | . . ~ ~ ~ ~ ~ ~ | ~ ~ . . . . . . |
        // | . . ~ ~ ~ ~ ~ ~ | ~ ~ . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // +-----------------+-----------------+
        let tilde = Self::empty()
            .or(nw.up(2).left(2))
            .or(ne.up(2).right(6))
            .or(sw.down(6).left(2))
            .or(se.down(6).right(6));
        let w = tilde.step2(rule);

        // +-----------------+-----------------+
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . ~ ~ | ~ ~ ~ ~ ~ ~ . . |
        // | . . . . . . ~ ~ | ~ ~ ~ ~ ~ ~ . . |
        // | . . . . . . ~ ~ | x x x x ~ ~ . . |
        // | . . . . . . ~ ~ | x x x x ~ ~ . . |
        // | . . . . . . ~ ~ | x x x x ~ ~ . . |
        // | . . . . . . ~ ~ | x x x x ~ ~ . . |
        // |-----------------+-----------------|
        // | . . . . . . ~ ~ | ~ ~ ~ ~ ~ ~ . . |
        // | . . . . . . ~ ~ | ~ ~ ~ ~ ~ ~ . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // +-----------------+-----------------+
        let tilde = Self::empty()
            .or(nw.up(2).left(6))
            .or(ne.up(2).right(2))
            .or(sw.down(6).left(6))
            .or(se.down(6).right(2));
        let x = tilde.step2(rule);

        // +-----------------+-----------------+
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . ~ ~ ~ ~ ~ ~ | ~ ~ . . . . . . |
        // | . . ~ ~ ~ ~ ~ ~ | ~ ~ . . . . . . |
        // |-----------------+-----------------|
        // | . . ~ ~ y y y y | ~ ~ . . . . . . |
        // | . . ~ ~ y y y y | ~ ~ . . . . . . |
        // | . . ~ ~ y y y y | ~ ~ . . . . . . |
        // | . . ~ ~ y y y y | ~ ~ . . . . . . |
        // | . . ~ ~ ~ ~ ~ ~ | ~ ~ . . . . . . |
        // | . . ~ ~ ~ ~ ~ ~ | ~ ~ . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // +-----------------+-----------------+
        let tilde = Self::empty()
            .or(nw.up(6).left(2))
            .or(ne.up(6).right(6))
            .or(sw.down(2).left(2))
            .or(se.down(2).right(6));
        let y = tilde.step2(rule);

        // +-----------------+-----------------+
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . ~ ~ | ~ ~ ~ ~ ~ ~ . . |
        // | . . . . . . ~ ~ | ~ ~ ~ ~ ~ ~ . . |
        // |-----------------+-----------------|
        // | . . . . . . ~ ~ | z z z z ~ ~ . . |
        // | . . . . . . ~ ~ | z z z z ~ ~ . . |
        // | . . . . . . ~ ~ | z z z z ~ ~ . . |
        // | . . . . . . ~ ~ | z z z z ~ ~ . . |
        // | . . . . . . ~ ~ | ~ ~ ~ ~ ~ ~ . . |
        // | . . . . . . ~ ~ | ~ ~ ~ ~ ~ ~ . . |
        // | . . . . . . . . | . . . . . . . . |
        // | . . . . . . . . | . . . . . . . . |
        // +-----------------+-----------------+
        let tilde = Self::empty()
            .or(nw.up(6).left(6))
            .or(ne.up(6).right(2))
            .or(sw.down(2).left(6))
            .or(se.down(2).right(2));
        let z = tilde.step2(rule);

        Self::empty()
            .or(w.up(2).left(2))
            .or(x.up(2).right(2))
            .or(y.down(2).left(2))
            .or(z.down(2).right(2))
    }

    const fn and(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    const fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    const fn xor(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }

    const fn neighbors(&self) -> [Bool64; 8] {
        [
            self.up(1).0,
            self.down(1).0,
            self.left(1).0,
            self.right(1).0,
            self.up(1).left(1).0,
            self.up(1).right(1).0,
            self.down(1).left(1).0,
            self.down(1).right(1).0,
        ]
    }

    const fn up(&self, steps: u8) -> Self {
        if steps == 0 {
            *self
        } else {
            Self((self.0 << 8) & !BORDER_BOTTOM).up(steps - 1)
        }
    }

    const fn down(&self, steps: u8) -> Self {
        if steps == 0 {
            *self
        } else {
            Self((self.0 >> 8) & !BORDER_TOP).down(steps - 1)
        }
    }

    const fn left(&self, steps: u8) -> Self {
        if steps == 0 {
            *self
        } else {
            Self((self.0 << 1) & !BORDER_RIGHT).left(steps - 1)
        }
    }

    const fn right(&self, steps: u8) -> Self {
        if steps == 0 {
            *self
        } else {
            Self((self.0 >> 1) & !BORDER_LEFT).right(steps - 1)
        }
    }
}

/// [half adder]
///
/// [half adder]: https://en.wikipedia.org/wiki/Adder_(electronics)#Half_adder
const fn half_adder(a: Bool64, b: Bool64) -> (Bool64, Bool64) {
    (a ^ b, a & b)
}

struct Adder {
    digits: [Bool64; 4],
}

impl Adder {
    const fn new() -> Self {
        Self { digits: [FALSE; 4] }
    }

    const fn add_all(self, inputs: &[Bool64]) -> Self {
        match inputs {
            &[] => self,
            &[head, ref tail @ ..] => self.add(head).add_all(tail),
        }
    }

    const fn add(self, input: Bool64) -> Self {
        let [b0, b1, b2, b3] = self.digits;

        // add the first digit to the input
        let (q0, carry) = half_adder(b0, input);
        // add the next digit to the previous carry
        let (q1, carry) = half_adder(b1, carry);
        // add the next digit to the previous carry
        let (q2, carry) = half_adder(b2, carry);
        // saturating add the final digit to the previous carry
        let q3 = b3 | carry;

        Self {
            digits: [q0, q1, q2, q3],
        }
    }

    #[rustfmt::skip]
    const fn sum(self) -> [Bool64; 9] {
        let [b0, b1, b2, b3] = self.digits;

        let zero  = !b3 & !b2 & !b1 & !b0;
        let one   = !b3 & !b2 & !b1 &  b0;
        let two   = !b3 & !b2 &  b1 & !b0;
        let three = !b3 & !b2 &  b1 &  b0;
        let four  = !b3 &  b2 & !b1 & !b0;
        let five  = !b3 &  b2 & !b1 &  b0;
        let six   = !b3 &  b2 &  b1 & !b0;
        let seven = !b3 &  b2 &  b1 &  b0;
        let eight =  b3 & !b2 & !b1 & !b0;

        [zero, one, two, three, four, five, six, seven, eight]
    }
}

const fn make_rule(neighbors: &[u8]) -> [Bool64; 9] {
    match neighbors {
        &[] => [FALSE; 9],
        &[head, ref tail @ ..] => {
            let mut result = make_rule(tail);
            result[head as usize] = TRUE;
            result
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rule {
    birth: [Bool64; 9],
    survival: [Bool64; 9],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram() {
        let buckets = Adder::new()
            .add(0x___0000000F00000000)
            .add(0x___000000FFF0000000)
            .add(0x___00000FFFFF000000)
            .add(0x___0000FFFFFFF00000)
            .add(0x___000FFFFFFFFF0000)
            .add(0x___00FFFFFFFFFFF000)
            .add(0x___0FFFFFFFFFFFFF00)
            .add(0x___FFFFFFFFFFFFFFF0)
            .sum();

        assert_eq!(0x_0000000F00000000, buckets[8]);
        assert_eq!(0x_000000F0F0000000, buckets[7]);
        assert_eq!(0x_00000F000F000000, buckets[6]);
        assert_eq!(0x_0000F00000F00000, buckets[5]);
        assert_eq!(0x_000F0000000F0000, buckets[4]);
        assert_eq!(0x_00F000000000F000, buckets[3]);
        assert_eq!(0x_0F00000000000F00, buckets[2]);
        assert_eq!(0x_F0000000000000F0, buckets[1]);
        assert_eq!(0x_000000000000000F, buckets[0]);
    }
}
