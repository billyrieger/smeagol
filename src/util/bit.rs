// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::Grid2;

use std::{
    default::Default,
    fmt::Debug,
    hash::Hash,
    ops::{BitAnd, BitOr, BitXor, Not},
};

pub trait BitSquare:
    Sized
    + Copy
    + Eq
    + Hash
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Not<Output = Self>
{
    type Quadrant: BitSquare;
    const SIDE_LEN: u8;
    const LOG_SIDE_LEN: u8;

    fn zero() -> Self;
    fn get_bit(&self, index: u32) -> bool;
    fn set_bit(&self, index: u32);
    fn unset_bit(&self, index: u32);
    fn count_ones(&self) -> u32;
    fn moore_neighborhood(&self) -> [Self; 8];
    fn from_parts(parts: Grid2<Self::Quadrant>) -> Self;
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Bit4x4(u16);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Bit8x8(u64);

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
    ( $ty:ident , $op:ident , $fn:ident , $symbol:tt ) => {
        impl $op<$ty> for $ty {
            type Output = $ty;

            fn $fn(self, rhs: $ty) -> $ty {
                $ty(self.0 $symbol rhs.0)
            }
        }
    };
}

macro_rules! bit_traits {
    ( $ty:ident ) => {
        unary_not!($ty);
        binary_op!($ty, BitAnd, bitand, &);
        binary_op!($ty, BitOr, bitor, |);
        binary_op!($ty, BitXor, bitxor, ^);
    };
}

bit_traits!(Bit4x4);
bit_traits!(Bit8x8);
