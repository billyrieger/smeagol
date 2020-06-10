// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod grid;
pub use grid::Grid2;

use std::{
    default::Default,
    fmt::Debug,
    hash::Hash,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

mod structs {
    use derive_more::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, From, Into, Not, Shl,
        ShlAssign, Shr, ShrAssign,
    };
    use packed_simd::u16x16;

    #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
    #[derive(BitAnd, BitOr, BitXor, From, Into, Not, Shl, Shr)]
    #[derive(BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign)]
    pub struct Bit4x4(u16);

    #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
    #[derive(BitAnd, BitOr, BitXor, From, Into, Not, Shl, Shr)]
    #[derive(BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign)]
    pub struct Bit8x8(u64);

    #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
    #[derive(BitAnd, BitOr, BitXor, From, Into, Not, Shl, Shr)]
    #[derive(BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign)]
    pub struct Bit16x16(u16x16);
}

pub use structs::*;

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
    + BitAndAssign
    + BitOrAssign
    + BitXorAssign
    + Not
{
    const SIDE_LEN: u8;

    fn moore_neighborhood(&self) -> [Self; 8] {
        unimplemented!()
    }

    fn count_true(&self) -> u32 {
        unimplemented!()
    }
}

impl BitMatrix for Bit4x4 {
    const SIDE_LEN: u8 = 4;

    fn moore_neighborhood(&self) -> [Self; 8] {
        [
            *self >> 5,
            *self >> 4,
            *self >> 3,
            *self >> 1,
            *self << 1,
            *self << 3,
            *self << 4,
            *self << 5,
        ]
    }
}

impl BitMatrix for Bit8x8 {
    const SIDE_LEN: u8 = 8;

    fn moore_neighborhood(&self) -> [Self; 8] {
        [
            *self >> 9,
            *self >> 8,
            *self >> 7,
            *self >> 1,
            *self << 1,
            *self << 7,
            *self << 8,
            *self << 9,
        ]
    }
}

impl BitMatrix for Bit16x16 {
    const SIDE_LEN: u8 = 16;

    fn moore_neighborhood(&self) -> [Self; 8] {
        let up = || {
            todo!()
        };

        let down = |grid: Self| {
            todo!()
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
}
