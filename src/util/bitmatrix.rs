// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::Grid2;

// traits from std
use std::{
    default::Default,
    fmt::Debug,
    hash::Hash,
    ops::{BitAnd, BitOr, BitXor, Not},
};

use packed_simd::u16x16;

macro_rules! unary_not {
    ( $ty:ident ) => {
        impl Not for $ty {
            type Output = $ty;

            fn not(self) -> $ty {
                $ty(!self.0)
            }
        }
    };
}

macro_rules! binary_op {
    ( $ty:ident , $op:ident , $fn:ident , $ident:tt ) => {
        impl $op<$ty> for $ty {
            type Output = $ty;

            fn $fn(self, rhs: $ty) -> $ty {
                $ty(self.0 $ident rhs.0)
            }
        }
    };
}

unary_not!(Bit8x8);
binary_op!(Bit8x8, BitAnd, bitand, &);
binary_op!(Bit8x8, BitOr, bitor, |);
binary_op!(Bit8x8, BitXor, bitxor, ^);

unary_not!(Bit16x16);
binary_op!(Bit16x16, BitAnd, bitand, &);
binary_op!(Bit16x16, BitOr, bitor, |);
binary_op!(Bit16x16, BitXor, bitxor, ^);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Bit8x8(pub u64);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Bit16x16(pub u16x16);

impl Bit16x16 {
    pub const SIDE_LEN: u8 = 16;

    pub fn from_parts(grid: Grid2<Bit8x8>) -> Self {
        let [nw, ne, sw, se]: [[u8; 8]; 4] = [
            grid.nw.0.to_be_bytes(),
            grid.ne.0.to_be_bytes(),
            grid.sw.0.to_be_bytes(),
            grid.se.0.to_be_bytes(),
        ];

        let f = u16::from_be_bytes;

        Bit16x16(u16x16::new(
            f([nw[0], ne[0]]),
            f([nw[1], ne[1]]),
            f([nw[2], ne[2]]),
            f([nw[3], ne[3]]),
            f([nw[4], ne[4]]),
            f([nw[5], ne[5]]),
            f([nw[6], ne[6]]),
            f([nw[7], ne[7]]),
            f([sw[0], se[0]]),
            f([sw[1], se[1]]),
            f([sw[2], se[2]]),
            f([sw[3], se[3]]),
            f([sw[4], se[4]]),
            f([sw[5], se[5]]),
            f([sw[6], se[6]]),
            f([sw[7], se[7]]),
        ))
    }

    pub fn moore_neighborhood(&self) -> [Self; 8] {
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

    pub fn zero() -> Self {
        Self([0; 16].into())
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
