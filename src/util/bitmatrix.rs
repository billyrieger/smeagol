// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// traits from std
use std::{
    default::Default,
    fmt::Debug,
    hash::Hash,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

/// These derive macro imports don't clash with the trait imports from std above
use derive_more::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, From, Into, Not,
};

use packed_simd::{shuffle, u16x16};

pub trait BitMatrix:
    Clone
    + Copy
    + Debug
    + Default
    + Eq
    + Hash
    + PartialEq
    + BitAnd
    + BitOr
    + BitXor
    + Not
    + BitAndAssign
    + BitOrAssign
    + BitXorAssign
{
    const SIDE_LEN: u8;

    fn moore_neighborhood(&self) -> [Self; 8];

    fn count(&self) -> u32;
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
// foo
#[derive(BitAnd, BitOr, BitXor, From, Into, Not)]
// foo
#[derive(BitAndAssign, BitOrAssign, BitXorAssign)]
pub struct Bit4x4(u16);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
// foo
#[derive(BitAnd, BitOr, BitXor, From, Into, Not)]
// foo
#[derive(BitAndAssign, BitOrAssign, BitXorAssign)]
pub struct Bit8x8(u64);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
// foo
#[derive(BitAnd, BitOr, BitXor, From, Into, Not)]
// foo
#[derive(BitAndAssign, BitOrAssign, BitXorAssign)]
pub struct Bit16x16(u16x16);

impl BitMatrix for Bit4x4 {
    const SIDE_LEN: u8 = 4;

    fn count(&self) -> u32 {
        0
    }

    fn moore_neighborhood(&self) -> [Self; 8] {
        [
            Self(self.0 >> 5),
            Self(self.0 >> 4),
            Self(self.0 >> 3),
            Self(self.0 >> 1),
            Self(self.0 << 1),
            Self(self.0 << 3),
            Self(self.0 << 4),
            Self(self.0 << 5),
        ]
    }
}

impl BitMatrix for Bit8x8 {
    const SIDE_LEN: u8 = 8;

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

    fn count(&self) -> u32 {
        0
    }

    fn moore_neighborhood(&self) -> [Self; 8] {
        let north = |matrix: Self| -> Self {
            let x: [u16; 16] = matrix.0.into();
            Self(
                [
                    x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8], x[9], x[10], x[11],
                    x[12], x[13], x[14], x[15],
                ]
                .into(),
            )
        };

        let up = |matrix: Self| {
            Self(shuffle!(
                matrix.0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0]
            ))
        };
        let down = |grid: Self| {
            Self(shuffle!(
                grid.0,
                [15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
            ))
        };
        let left = |grid: Self| Self(grid.0 << 1);
        let right = |grid: Self| Self(grid.0 >> 1);

        [
            down(right(*self)),
            down(*self),
            down(left(*self)),
            right(*self),
            left(*self),
            up(right(*self)),
            up(*self),
            up(left(*self)),
        ]
    }
}
