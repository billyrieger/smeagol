// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::mem::MaybeUninit;
use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};
use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};

use derive_more as dm;

// Derive macros from the standard library.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// `derive_more` macros.
#[derive(dm::Add, dm::Sub)]
pub struct Vec2 {
    pub x: i64,
    pub y: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Grid2<T> {
    pub nw: T,
    pub ne: T,
    pub sw: T,
    pub se: T,
}

pub trait ToGrid<T> {
    fn to_grid(self) -> Grid2<T>;
}

impl<T> ToGrid<T> for [T; 4] {
    fn to_grid(self) -> Grid2<T> {
        Grid2::from_array(self)
    }
}

pub enum Dir {
    North,
    South,
    East,
    West,
}

pub enum OrdinalDir {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
}

impl Dir {
    pub fn unit_vec(&self) -> Vec2 {
        match self {
            Dir::North => Vec2::new(0, 1),
            Dir::South => Vec2::new(0, -1),
            Dir::East => Vec2::new(1, 0),
            Dir::West => Vec2::new(-1, 0),
        }
    }
}

impl Vec2 {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}

impl<T> Grid2<T> {
    // pub fn new(nw: T, ne: T, sw: T, se: T) -> Self {
    //     Self { nw, ne, sw, se }
    // }

    pub fn from_array([nw, ne, sw, se]: [T; 4]) -> Self {
        Self { nw, ne, sw, se }
    }

    pub fn to_array(self) -> [T; 4] {
        [self.nw, self.ne, self.sw, self.se]
    }
}

// impl<T> From<[T; 4]> for Grid2<T> {
//     fn from(array: [T; 4]) -> Self {
//         Self::from_array(array)
//     }
// }

pub trait ArrayConcatExt<T, const N: usize> {
    fn array_concat<const M: usize>(self, other: [T; M]) -> [T; N + M];
}

impl<T, const N: usize> ArrayConcatExt<T, N> for [T; N] {
    fn array_concat<const M: usize>(self, other: [T; M]) -> [T; N + M] {
        let mut result = MaybeUninit::uninit_array();
        *result.split_array_mut().0 = self.map(MaybeUninit::new);
        *result.rsplit_array_mut().1 = other.map(MaybeUninit::new);
        // SAFETY: the length of the result array is N+M. We initialized the
        // first N elements using `self` and the last M elements using `other`,
        // so all the elements have been initialized.
        unsafe { MaybeUninit::array_assume_init(result) }
    }
}

pub trait ArrayUnzipExt<T, U, const N: usize> {
    fn unzip_array(self) -> ([T; N], [U; N]);
}

impl<T, U, const N: usize> ArrayUnzipExt<T, U, N> for [(T, U); N] {
    fn unzip_array(self) -> ([T; N], [U; N]) {
        todo!()
    }
}

pub trait BitGrid:
    Sized
    + Copy
    + BitAnd<Self, Output = Self>
    + BitOr<Self, Output = Self>
    + BitXor<Self, Output = Self>
{
    const ROWS: usize;
    const COLS: usize;

    fn count_ones(&self) -> u32;
    fn get(&self, row: usize, col: usize) -> Option<bool>;
    fn set(self, row: usize, col: usize, value: bool) -> Option<Self>;
    fn shift(self, dir: Dir) -> Self;
}

trait Num:
    Sized
    + Copy
    + Ord
    + BitAnd<Self, Output = Self>
    + BitOr<Self, Output = Self>
    + BitXor<Self, Output = Self>
    + Not<Output = Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
{
    const ZERO: Self;
    const ONE: Self;

    fn count_ones(&self) -> u32;
}

macro_rules! impl_num {
    ( $int:ty ) => {
        impl Num for $int {
            const ZERO: Self = 0;
            const ONE: Self = 1;
            fn count_ones(&self) -> u32 {
                <$int>::count_ones(*self)
            }
        }
    };
}

impl_num!(u8);
impl_num!(u16);
impl_num!(u32);
impl_num!(u64);
impl_num!(u128);

impl<T, const LANES: usize> BitGrid for Simd<T, LANES>
where
    Self: BitAnd<Self, Output = Self>
        + BitOr<Self, Output = Self>
        + BitXor<Self, Output = Self>
        + Shl<Self, Output = Self>
        + Shr<Self, Output = Self>,
    T: SimdElement + Num,
    LaneCount<LANES>: SupportedLaneCount,
{
    const ROWS: usize = LANES;
    const COLS: usize = std::mem::size_of::<T>() * 8;

    fn count_ones(&self) -> u32 {
        self.to_array().map(|row| row.count_ones()).iter().sum()
    }

    fn get(&self, row: usize, col: usize) -> Option<bool> {
        (row < Self::ROWS && col < Self::COLS).then(|| {
            let bitmask = T::ONE << (Self::COLS - col - 1);
            self.as_array()[row] & bitmask > T::ZERO
        })
    }

    fn set(mut self, row: usize, col: usize, value: bool) -> Option<Self> {
        (row < Self::ROWS && col < Self::COLS).then(|| {
            let bitmask = T::ONE << (Self::COLS - col - 1);
            let row = &mut self.as_mut_array()[row];
            *row = if value {
                *row | bitmask
            } else {
                *row & !bitmask
            };
            self
        })
    }

    fn shift(self, dir: Dir) -> Self {
        match dir {
            Dir::North => self.rotate_lanes_left::<1>(),
            Dir::South => self.rotate_lanes_right::<1>(),
            Dir::East => self >> Self::splat(T::ONE),
            Dir::West => self << Self::splat(T::ONE),
        }
    }
}
