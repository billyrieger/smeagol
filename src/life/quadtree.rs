// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::prelude::*;

use crate::{
    life::Cell,
    util::{Bit8x8, Grid2},
};

use std::{
    convert::TryFrom,
    ops::{Add, Div, Mul, Sub},
    option::NoneError,
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Length(u64);

impl Length {
    pub const MAX: Length = Self(1u64 << 63);
    pub const HALF_MAX: Length = Self(1u64 << 62);
}

impl Into<u64> for Length {
    fn into(self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for Length {
    type Error = NoneError;

    fn try_from(val: u64) -> Result<Length> {
        if val <= Length::MAX.0 {
            Ok(Length(val))
        } else {
            None?
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Coord(i64);

impl Coord {
    pub const MIN: Coord = Self(-(1i64 << 62));
    pub const MAX: Coord = Self((1i64 << 62) - 1);
}

#[macro_export]
macro_rules! impl_binop {
    ( $op:ident , $func:ident , $type:ident ) => {
        impl $op<$type> for $type {
            type Output = $type;

            fn $func(self, other: $type) -> $type {
                $type(self.0 + other.0)
            }
        }

        impl<'lhs> $op<$type> for &'lhs $type {
            type Output = $type;

            fn $func(self, other: $type) -> $type {
                $type(self.0 + other.0)
            }
        }

        impl<'rhs> $op<&'rhs $type> for $type {
            type Output = $type;

            fn $func(self, other: &'rhs $type) -> $type {
                $type(self.0 + other.0)
            }
        }

        impl<'lhs, 'rhs> $op<&'rhs $type> for &'lhs $type {
            type Output = $type;

            fn $func(self, other: &'rhs $type) -> $type {
                $type(self.0 + other.0)
            }
        }
    };
}

impl_binop!(Add, add, Coord);
impl_binop!(Sub, sub, Coord);
impl_binop!(Mul, mul, Coord);
impl_binop!(Div, div, Coord);

impl Into<i64> for Coord {
    fn into(self) -> i64 {
        self.0
    }
}

impl TryFrom<i64> for Coord {
    type Error = NoneError;

    fn try_from(val: i64) -> Result<Coord> {
        if Coord::MIN.0 <= val && val <= Coord::MAX.0 {
            Ok(Coord(val))
        } else {
            None?
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub const ORIGIN: Self = Self::new(0, 0);

    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub const fn offset(&self, dx: i64, dy: i64) -> Position {
        Self::new(self.x + dx, self.y + dy)
    }

    pub const fn relative_to(&self, other: Position) -> Position {
        self.offset(-other.x, -other.y)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Id {
    index: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Leaf {
    pub alive: Bit8x8,
}

impl Leaf {
    const MIN: i64 = -4;
    const MAX: i64 = 3;
    const SIDE_LEN: i64 = 8;

    pub const fn new(alive: Bit8x8) -> Self {
        Self { alive }
    }

    pub fn alive_cells(&self) -> Vec<Position> {
        let mut result = Vec::new();

        if self.alive == Bit8x8::default() {
            return result;
        }

        let mut bits: u64 = self.alive.into();
        let mut reverse_index: usize = 0;

        while bits > 0 {
            let n_zeros = bits.leading_zeros() as usize;

            bits <<= n_zeros;
            reverse_index += n_zeros;

            result.push(self.idx_to_pos(63 - reverse_index));

            reverse_index += 1;
            bits <<= 1;
        }

        result
    }

    fn check_bounds(&self, pos: Position) -> bool {
        let x_ok = Self::MIN <= pos.x && pos.x <= Self::MAX;
        let y_ok = Self::MIN <= pos.y && pos.y <= Self::MAX;
        x_ok && y_ok
    }

    fn pos_to_idx(&self, pos: Position) -> usize {
        (Self::SIDE_LEN * (Self::MAX - pos.y) + (Self::MAX - pos.x)) as usize
    }

    fn idx_to_pos(&self, index: usize) -> Position {
        let index = index as i64;
        let y = Self::MAX - index / Self::SIDE_LEN;
        let x = Self::MAX - index % Self::SIDE_LEN;
        Position::new(x, y)
    }

    pub fn population(&self) -> u128 {
        u128::from(self.alive.0.count_ones())
    }

    pub fn get_cell(&self, pos: Position) -> Option<Cell> {
        if self.check_bounds(pos) {
            None
        } else {
            let index = self.pos_to_idx(pos);
            if self.alive.0 & (1 << index) == 0 {
                Some(Cell::Dead)
            } else {
                Some(Cell::Alive)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Branch {
    pub children: Grid2<Id>,
    pub side_len: u64,
    pub population: u128,
}

#[derive(Clone, Copy, Debug)]
pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

impl Node {
    pub fn population(&self) -> u128 {
        match self {
            Node::Leaf(leaf) => leaf.population(),
            Node::Branch(branch) => branch.population,
        }
    }
}
