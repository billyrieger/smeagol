// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod rule;
pub use rule::Rule;

use crate::{store::Leaf, util::Grid2};

use derive_more::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};
use packed_simd::{shuffle, u16x16};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[derive(BitAnd, BitOr, BitXor, Shl, Shr, Not)]
#[derive(BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign)]
struct Bool16x16(u16x16);

impl Bool16x16 {
    fn from_leaves(leaves: Grid2<Leaf>) -> Self {
        let [nw, ne, sw, se]: [[u8; 8]; 4] = [
            leaves.nw.alive.to_be_bytes(),
            leaves.ne.alive.to_be_bytes(),
            leaves.sw.alive.to_be_bytes(),
            leaves.se.alive.to_be_bytes(),
        ];

        Self(u16x16::new(
            u16::from_be_bytes([nw[0], ne[0]]),
            u16::from_be_bytes([nw[1], ne[1]]),
            u16::from_be_bytes([nw[2], ne[2]]),
            u16::from_be_bytes([nw[3], ne[3]]),
            u16::from_be_bytes([nw[4], ne[4]]),
            u16::from_be_bytes([nw[5], ne[5]]),
            u16::from_be_bytes([nw[6], ne[6]]),
            u16::from_be_bytes([nw[7], ne[7]]),
            u16::from_be_bytes([sw[0], se[0]]),
            u16::from_be_bytes([sw[1], se[1]]),
            u16::from_be_bytes([sw[2], se[2]]),
            u16::from_be_bytes([sw[3], se[3]]),
            u16::from_be_bytes([sw[4], se[4]]),
            u16::from_be_bytes([sw[5], se[5]]),
            u16::from_be_bytes([sw[6], se[6]]),
            u16::from_be_bytes([sw[7], se[7]]),
        ))
    }

    fn moore_neighborhood(&self) -> [Self; 8] {
        let up = |grid: Self| -> Self {
            Self(shuffle!(
                grid.0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0]
            ))
        };

        let down = |grid: Self| {
            Self(shuffle!(
                grid.0,
                [15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
            ))
        };

        let left = |grid: Self| Self(grid.0.rotate_left(u16x16::splat(1)));

        let right = |grid: Self| Self(grid.0.rotate_right(u16x16::splat(1)));

        [
            up(*self),
            down(*self),
            left(*self),
            right(*self),
            up(left(*self)),
            up(right(*self)),
            down(left(*self)),
            down(right(*self)),
        ]
    }

    fn center_leaf(&self) -> Leaf {
        let grid: [u16; 16] = self.0.into();

        let middle = |row: u16| (row >> 4) as u8;

        Leaf {
            alive: u64::from_be_bytes([
                middle(grid[4]),
                middle(grid[5]),
                middle(grid[6]),
                middle(grid[7]),
                middle(grid[8]),
                middle(grid[9]),
                middle(grid[10]),
                middle(grid[11]),
            ]),
        }
    }

    fn half_adder(&mut self, addend: Self) -> Self {
        let carry = Self(self.0 & addend.0);
        self.0 ^= addend.0;
        carry
    }
}

pub struct B3S23;

impl Rule for B3S23 {
    type Leaf = Leaf;

    fn evolve(&self, grid: Grid2<Leaf>, steps: u8) -> Leaf {
        assert!(steps <= 4);

        let step_once = |alive: Bool16x16| -> Bool16x16 {
            let [mut d2, mut d1, mut d0]: [Bool16x16; 3] = Default::default();

            for &addend in &alive.moore_neighborhood() {
                let carry0 = d0.half_adder(addend);
                let carry1 = d1.half_adder(carry0);
                d2 |= carry1;
            }

            // two is 010 is binary
            let sum_is_two: Bool16x16 = !d2 & d1 & !d0;

            // three is 011 in binary
            let sum_is_three = !d2 & d1 & d0;

            sum_is_three | (alive & sum_is_two)
        };

        let mut result = Bool16x16::from_leaves(grid);
        for _ in 0..steps {
            result = step_once(result);
        }
        result.center_leaf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glider() {
        let rule = B3S23;

        let nw = Leaf::new(u64::from_be_bytes([
            0b_00000000,
            0b_00000000,
            0b_00000000,
            0b_00000000,
            0b_00000100,
            0b_00000010,
            0b_00001110,
            0b_00000000,
        ]));
        let [ne, sw, se] = [Leaf::DEAD; 3];

        let grid = Grid2 { nw, ne, sw, se };

        let expected0 = Leaf::new(u64::from_be_bytes([
            0b_01000000,
            0b_00100000,
            0b_11100000,
            0b_00000000,
            0b_00000000,
            0b_00000000,
            0b_00000000,
            0b_00000000,
        ]));

        let expected4 = Leaf::new(expected0.alive >> 9);

        assert_eq!(rule.evolve(grid, 0), expected0);
        assert_eq!(rule.evolve(grid, 4), expected4);
    }
}
