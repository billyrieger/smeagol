// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::mem::MaybeUninit;

use derive_more as dm;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Grid2<T> {
    pub nw: T,
    pub ne: T,
    pub sw: T,
    pub se: T,
}

// Derive macros from the standard library.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// `derive_more` macros.
#[derive(dm::Add, dm::Sub)]
pub struct Vec2 {
    pub x: i64,
    pub y: i64,
}

pub enum Dir {
    North,
    South,
    East,
    West,
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

pub trait ToGrid<T> {
    fn to_grid(self) -> Grid2<T>;
}

impl<T> ToGrid<T> for [T; 4] {
    fn to_grid(self) -> Grid2<T> {
        Grid2::from_array(self)
    }
}

pub trait ArrayConcatExt<T, const N: usize> {
    fn array_concat<const M: usize>(self, other: [T; M]) -> [T; N + M];
}

impl<T, const N: usize> ArrayConcatExt<T, N> for [T; N] {
    fn array_concat<const M: usize>(self, other: [T; M]) -> [T; N + M] {
        let mut result = MaybeUninit::uninit_array();
        *result.split_array_mut().0 = self.map(MaybeUninit::new);
        *result.rsplit_array_mut().1 = other.map(MaybeUninit::new);
        // SAFETY: the length of the result array is N+M. We initialized the first N elements and then
        // the last M elements, so all the elements have been initialized.
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
