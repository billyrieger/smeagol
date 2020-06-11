// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::Grid2;

// traits from std
use std::{
    default::Default,
    fmt::Debug,
    hash::Hash,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

// these derive macro imports don't clash with the trait imports from std above
use derive_more::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, From, Into, Not, Shl,
    ShlAssign, Shr, ShrAssign,
};

use packed_simd::u16x16;

pub trait BitMatrix:
    Clone
    + Copy
    + Eq
    + Hash
    + PartialEq
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Not<Output = Self>
    + BitAndAssign
    + BitOrAssign
    + BitXorAssign
{
    const SIDE_LEN: u8;

    fn zero() -> Self;

    fn moore_neighborhood(&self) -> [Self; 8];

    fn count(&self) -> u32;
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
// foo
#[derive(BitAnd, BitOr, BitXor, From, Into, Not, Shl, Shr)]
// foo
#[derive(BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign)]
pub struct Bit4x4(u16);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
// foo
#[derive(BitAnd, BitOr, BitXor, From, Into, Not, Shl, Shr)]
// foo
#[derive(BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign)]
pub struct Bit8x8(u64);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
// foo
#[derive(BitAnd, BitOr, BitXor, From, Into, Not, Shl, Shr)]
// foo
#[derive(BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign)]
pub struct Bit16x16(u16x16);

impl BitMatrix for Bit4x4 {
    const SIDE_LEN: u8 = 4;

    fn zero() -> Self {
        Self(0)
    }

    fn count(&self) -> u32 {
        0
    }

    fn moore_neighborhood(&self) -> [Self; 8] {
        let x = *self;
        [
            x >> 5,
            x >> 4,
            x >> 3,
            x >> 1,
            x << 1,
            x << 3,
            x << 4,
            x << 5,
        ]
    }
}

impl BitMatrix for Bit8x8 {
    const SIDE_LEN: u8 = 8;

    fn zero() -> Self {
        Self(0)
    }

    fn count(&self) -> u32 {
        0
    }

    fn moore_neighborhood(&self) -> [Self; 8] {
        [
            Self(self.0 >> 9),
            Self(self.0 >> 8),
            Self(self.0 >> 7),
            Self(self.0 >> 1),
            Self(self.0 << 1),
            Self(self.0 << 7),
            Self(self.0 << 8),
            Self(self.0 << 9),
        ]
    }
}

impl BitMatrix for Bit16x16 {
    const SIDE_LEN: u8 = 16;

    fn zero() -> Self {
        Self([0; 16].into())
    }

    fn count(&self) -> u32 {
        0
    }

    fn moore_neighborhood(&self) -> [Self; 8] {
        let array: [u16; 16] = self.0.into();

        let mut north = array;
        north.rotate_right(1);
        let north = Self(north.into());

        let mut south = array;
        south.rotate_left(1);
        let south = Self(south.into());

        let west = Self(self.0 >> 1);
        let east = Self(self.0 << 1);

        let northwest = Self(north.0 >> 1);
        let southwest = Self(south.0 >> 1);

        let northeast = Self(north.0 << 1);
        let southeast = Self(south.0 << 1);

        [
            northwest, north, northeast, west, east, southwest, south, southeast,
        ]
    }
}

pub fn center(matrix: Bit16x16) -> Bit8x8 {
    let grid: [u16; 16] = matrix.0.into();

    let middle = |row: u16| (row >> 4) as u8;

    Bit8x8(u64::from_be_bytes([
        middle(grid[4]),
        middle(grid[5]),
        middle(grid[6]),
        middle(grid[7]),
        middle(grid[8]),
        middle(grid[9]),
        middle(grid[10]),
        middle(grid[11]),
    ]))
}

pub fn combine(grid: Grid2<Bit8x8>) -> Bit16x16 {
    let [nw, ne, sw, se]: [[u8; 8]; 4] = [
        grid.nw.0.to_be_bytes(),
        grid.ne.0.to_be_bytes(),
        grid.sw.0.to_be_bytes(),
        grid.se.0.to_be_bytes(),
    ];

    Bit16x16(u16x16::new(
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

#[cfg(test)]
mod tests {
    use super::*;

    fn evolve<B>(grid: B, steps: u8) -> B
    where
        B: BitMatrix + Debug,
    {
        assert!(steps <= B::SIDE_LEN / 4);

        let half_adder = |sum: B, addend: B| (sum ^ addend, sum & addend);

        let step_once = |alive: B| -> B {
            let mut sum: [B; 3] = [B::zero(); 3];

            for &addend in &alive.moore_neighborhood() {
                let (sum0, carry) = half_adder(sum[0], addend);
                let (sum1, carry) = half_adder(sum[1], carry);
                let sum2 = sum[2] | carry;

                sum = [sum0, sum1, sum2];
            }

            // two is 010 is binary
            let sum_is_two = !sum[2] & sum[1] & !sum[0];

            // three is 011 is binary
            let sum_is_three = !sum[2] & sum[1] & sum[0];

            sum_is_three | (alive & sum_is_two)
        };

        let mut result = grid;
        for _ in 0..steps {
            result = step_once(result);
        }
        result
    }

    #[test]
    fn blinker4x4() {
        let vertical: Bit4x4 = 0b_0010_0010_0010_0000.into();
        let horizontal: Bit4x4 = 0b_0000_0111_0000_0000.into();

        assert_eq!(evolve(vertical, 1), horizontal);
        assert_eq!(evolve(horizontal, 1), vertical);
    }

    #[test]
    fn blinker8x8() {
        let vertical: Bit8x8 = 0x_00_00_08_08_08_00_00_00.into();
        let horizontal: Bit8x8 = 0x_00_00_00_1C_00_00_00_00.into();

        assert_eq!(evolve(vertical, 1), horizontal);
        assert_eq!(evolve(horizontal, 1), vertical);
    }

    #[test]
    fn blinker16x16() {
        let vertical: Bit16x16 =
            Bit16x16([0, 0, 0, 0, 0, 0, 0x08, 0x08, 0x08, 0, 0, 0, 0, 0, 0, 0_u16].into());
        let horizontal: Bit16x16 = Bit16x16(
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00,
            ]
            .into(),
        );

        assert_eq!(evolve(vertical, 1), horizontal);
        assert_eq!(evolve(horizontal, 1), vertical);
    }
}
