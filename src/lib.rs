// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(const_fn, const_if_match)]

pub mod grid;
pub mod node;
pub mod store;

use node::{Id, Level};

use thiserror::Error;

/// A runtime error.
#[derive(Debug, Error)]
pub enum Error {
    /// The step sisze was too large.
    #[error("step size {step:?} too large for node with level {level:?}")]
    StepOverflow { step: u64, level: Level },

    /// Increment error.
    #[error("cannot increment past the maximum level")]
    Increment,

    /// Id not found.
    #[error("id {0:?} not found")]
    IdNotFound(Id),

    /// Unbalanced.
    #[error("unbalanced")]
    Unbalanced,
}

/// A result.
pub type Result<T> = std::result::Result<T, Error>;

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
///
/// # Examples
///
/// ```
/// # use smeagol::Bool8x8;
/// // 0x00 | . . . . . . . . 0000 = 0, 0000 = 0
/// // 0x3C | . . # # # # . . 0011 = 3, 1100 = C
/// // 0x20 | . . # . . . . . 0010 = 2, 0000 = 0
/// // 0x38 | . . # # # . . . 0011 = 3, 1000 = 8
/// // 0x20 | . . # . . . . . 0010 = 2, 0000 = 0
/// // 0x20 | . . # . . . . . 0010 = 2, 0000 = 0
/// // 0x20 | . . # . . . . . 0010 = 2, 0000 = 0
/// // 0x00 | . . . . . . . . 0000 = 0, 0000 = 0
/// const UPPERCASE_F = Bool8x8(0x003C_2038_2020_2000);
/// let also_f = Bool8x8(u64::from_be_bytes([0x00, 0x3C, 0x20, 0x38, 0x20, 0x20, 0x20, 0x00]));
/// assert_eq!(UPPERCASE_F, also_f);
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bool8x8(pub u64);

/// The result of a bitwise sum of at most eight `Bool8x8`s.
///
/// `SumResult[n]` is `true` if and only if the sum was `n`.
type SumResult = [Bool8x8; 9];

impl Bool8x8 {
    /// The `Bool8x8` where all elements are `false`.
    pub const FALSE: Self = Self(0);

    /// The `Bool8x8` where all elements are `true`.
    pub const TRUE: Self = Self(u64::MAX);

    /// The `Bool8x8` where the top half is true.
    pub const NORTH: Self = Self(0xFFFF_FFFF_0000_0000);

    /// The `Bool8x8` where the bottom half is true.
    pub const SOUTH: Self = Self(0x0000_0000_FFFF_FFFF);

    /// The `Bool8x8` where the right half is true.
    pub const EAST: Self = Self(0x00FF_00FF_00FF_00FF);

    /// The `Bool8x8` where the left half is true.
    pub const WEST: Self = Self(0xFF00_FF00_FF00_FF00);

    /// The `Bool8x8` where the top-left quarter is true.
    pub const NORTHWEST: Self = Self(0xF0F0_F0F0_0000_0000);

    /// The `Bool8x8` where the top-right quarter is true.
    pub const NORTHEAST: Self = Self(0x0F0F_0F0F_0000_0000);

    /// The `Bool8x8` where the bottom-left quarter is true.
    pub const SOUTHWEST: Self = Self(0x0000_0000_F0F0_F0F0);

    /// The `Bool8x8` where the bottom-right quarter is true.
    pub const SOUTHEAST: Self = Self(0x0000_0000_0F0F_0F0F);

    /// The `Bool8x8` where the middle quarter is true.
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
            [] => result,
            &[head, ref tail @ ..] => Self::fold_or(result.or(head), tail),
        }
    }

    pub const fn fold_xor(result: Self, list: &[Self]) -> Self {
        match list {
            [] => result,
            &[head, ref tail @ ..] => Self::fold_xor(result.xor(head), tail),
        }
    }

    pub const fn all(list: &[Self]) -> Self {
        Self::fold_and(Self::TRUE, list)
    }

    pub const fn any(list: &[Self]) -> Self {
        Self::fold_or(Self::FALSE, list)
    }

    /// Performs bitwise boolean NOT.
    pub const fn not(&self) -> Self {
        Self(!self.0)
    }

    /// Shifts the `Bool8x8` to the left by the given number of steps.
    ///
    /// ```
    /// # use smeagol::Bool8x8;
    /// let uppercase_f = Bool8x8(0x003C_2038_2020_2000);
    /// // 0x00 | . . . . . . . .
    /// // 0x78 | . # # # # . . .
    /// // 0x40 | . # . . . . . .
    /// // 0x70 | . # # # . . . .
    /// // 0x40 | . # . . . . . .
    /// // 0x40 | . # . . . . . .
    /// // 0x40 | . # . . . . . .
    /// // 0x00 | . . . . . . . .
    /// let f_left = Bool8x8(0x0078_4070_4040_4000);
    /// assert_eq!(uppercase_f.left(1), f_left);
    /// ```
    pub const fn left(&self, steps: u8) -> Self {
        Self(self.0 << steps)
    }

    /// Shifts the `Bool8x8` to the right by the given number of steps.
    ///
    /// ```
    /// # use smeagol::Bool8x8;
    /// let uppercase_f = Bool8x8(0x003C_2038_2020_2000);
    /// // 0x00 | . . . . . . . .
    /// // 0x1E | . . . # # # # .
    /// // 0x10 | . . . # . . . .
    /// // 0x1C | . . . # # # . .
    /// // 0x10 | . . . # . . . .
    /// // 0x10 | . . . # . . . .
    /// // 0x10 | . . . # . . . .
    /// // 0x00 | . . . . . . . .
    /// let f_right = Bool8x8(0x001E_101C_1010_1000);
    /// assert_eq!(uppercase_f.right(1), f_right);
    /// ```
    pub const fn right(&self, steps: u8) -> Self {
        Self(self.0 >> steps)
    }

    /// Shifts the `Bool8x8` up by the given number of steps.
    ///
    /// ```
    /// # use smeagol::Bool8x8;
    /// let uppercase_f = Bool8x8(0x003C_2038_2020_2000);
    /// // 0x3C | . . # # # # . .
    /// // 0x20 | . . # . . . . .
    /// // 0x38 | . . # # # . . .
    /// // 0x20 | . . # . . . . .
    /// // 0x20 | . . # . . . . .
    /// // 0x20 | . . # . . . . .
    /// // 0x00 | . . . . . . . .
    /// // 0x00 | . . . . . . . .
    /// let f_up = Bool8x8(0x3C20_3820_2020_0000);
    /// assert_eq!(uppercase_f.up(1), f_up);
    /// ```
    pub const fn up(&self, steps: u8) -> Self {
        self.left(steps * 8)
    }

    /// Shifts the `Bool8x8` down by the given number of steps.
    ///
    /// ```
    /// # use smeagol::Bool8x8;
    /// let uppercase_f = Bool8x8(0x003C_2038_2020_2000);
    /// // 0x00 | . . . . . . . .
    /// // 0x00 | . . . . . . . .
    /// // 0x3C | . . # # # # . .
    /// // 0x20 | . . # . . . . .
    /// // 0x38 | . . # # # . . .
    /// // 0x20 | . . # . . . . .
    /// // 0x20 | . . # . . . . .
    /// // 0x20 | . . # . . . . .
    /// let f_down = Bool8x8(0x000_3C20_3820_2020);
    /// assert_eq!(uppercase_f.down(1), f_down);
    /// ```
    pub const fn down(&self, steps: u8) -> Self {
        self.right(steps * 8)
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

    const fn any_both(xs: &[Self], ys: &[Self]) -> Self {
        match (xs, ys) {
            (&[], &[]) => Self::TRUE,
            (&[], &[..]) => Self::FALSE,
            (&[..], &[]) => Self::FALSE,
            (&[x, ref xs @ ..], &[y, ref ys @ ..]) => Self::FALSE,
        }
    }

    const fn any_are_both_true(x: SumResult, y: SumResult) -> Self {
        let [ax, bx, cx, dx, ex, fx, gx, hx, ix] = x;
        let [ay, by, cy, dy, ey, fy, gy, hy, iy] = y;
        Self::any(&[
            ax.and(ay),
            bx.and(by),
            cx.and(cy),
            dx.and(dy),
            ex.and(ey),
            fx.and(fy),
            gx.and(gy),
            hx.and(hy),
            ix.and(iy),
        ])
        // Self::FALSE
        //     .or(ax.and(ay))
        //     .or(bx.and(by))
        //     .or(cx.and(cy))
        //     .or(dx.and(dy))
        //     .or(ex.and(ey))
        //     .or(fx.and(fy))
        //     .or(gx.and(gy))
        //     .or(hx.and(hy))
        //     .or(ix.and(iy))
    }
}

/// A description of how one state of a cellular automaton transitions into the next.
#[derive(Clone, Copy, Debug, Default)]
pub struct Rule {
    birth_neighbors: SumResult,
    survival_neighbors: SumResult,
}

impl Rule {
    /// Creates a new `Rule` using B/S notation.
    ///
    /// From [LifeWiki]:
    ///
    /// > The most common notation for rulestrings B{number list}/S{number list}; this is referred
    /// > to as "B/S notation", and is sometimes called the rulestring of the [cellular automaton]
    /// > in question. B (for birth) is a list of all the numbers of live neighbors that cause a
    /// > dead cell to come alive (be born); S (for survival) is a list of all the numbers of live
    /// > neighbors that cause a live cell to remain alive (survive).
    ///
    /// # Examples
    ///
    /// ```
    /// # use smeagol::Rule;
    /// // Conway's Game of Life: B3/S23
    /// let life = Rule::new(&[3], &[2, 3]);
    /// ```
    ///
    /// [LifeWiki]: https://www.conwaylife.com/wiki/Rulestring#B.2FS_notation
    pub const fn new(birth: &[usize], survival: &[usize]) -> Self {
        let empty = [Bool8x8::FALSE; 9];
        Self {
            birth_neighbors: Self::make_rule(empty, birth),
            survival_neighbors: Self::make_rule(empty, survival),
        }
    }

    /// Evolves a `Bool8x8` to its next state, treating `true` as alive and `false` as dead.
    pub const fn step(&self, cells: Bool8x8) -> Bool8x8 {
        let (alive, dead) = (cells, cells.not());

        let alive_neighbors = Bool8x8::sum(&[
            alive.up(1),
            alive.down(1),
            alive.left(1),
            alive.right(1),
            alive.up(1).left(1),
            alive.up(1).right(1),
            alive.down(1).left(1),
            alive.down(1).right(1),
        ]);

        //         let [ax, bx, cx, dx, ex, fx, gx, hx, ix] = x;
        //         let [ay, by, cy, dy, ey, fy, gy, hy, iy] = y;
        //         Self::FALSE
        //             .or(ax.and(ay))
        //             .or(bx.and(by))
        //             .or(cx.and(cy))
        //             .or(dx.and(dy))
        //             .or(ex.and(ey))
        //             .or(fx.and(fy))
        //             .or(gx.and(gy))
        //             .or(hx.and(hy))
        //             .or(ix.and(iy))
        //     }

        let born = Bool8x8::any_are_both_true(alive_neighbors, self.birth_neighbors);
        let survives = Bool8x8::any_are_both_true(alive_neighbors, self.survival_neighbors);

        dead.and(born).or(alive.and(survives))
    }

    const fn make_rule(result: SumResult, neighbors: &[usize]) -> SumResult {
        let t_ = Bool8x8::TRUE;
        let [a, b, c, d, e, f, g, h, i] = result;
        match neighbors {
            [] => result,
            [head, tail @ ..] => {
                let result = match head {
                    0 => [t_, b, c, d, e, f, g, h, i],
                    1 => [a, t_, c, d, e, f, g, h, i],
                    2 => [a, b, t_, d, e, f, g, h, i],
                    3 => [a, b, c, t_, e, f, g, h, i],
                    4 => [a, b, c, d, t_, f, g, h, i],
                    5 => [a, b, c, d, e, t_, g, h, i],
                    6 => [a, b, c, d, e, f, t_, h, i],
                    7 => [a, b, c, d, e, f, g, t_, i],
                    8 => [a, b, c, d, e, f, g, h, t_],
                    _ => result,
                };
                Self::make_rule(result, tail)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_partitions() {
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

        assert_eq!(center.up(1), north);
        assert_eq!(center.down(1), south);
        assert_eq!(center.right(1), east);
        assert_eq!(center.left(1), west);

        assert_eq!(center.up(2).left(3).down(1).right(3).down(2).up(1), center);
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

    #[test]
    fn blinker() {
        let life = Rule::new(&[3], &[2, 3]);

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x38 | . . # # # . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let blinker_horiz = Bool8x8(0x0000_0038_0000_0000);

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x10 | . . . # . . . .
        // 0x10 | . . . # . . . .
        // 0x10 | . . . # . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let blinker_vert = Bool8x8(0x0000_1010_1000_0000);

        assert_eq!(life.step(blinker_horiz), blinker_vert);
        assert_eq!(life.step(blinker_vert), blinker_horiz);
    }
}
