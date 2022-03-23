// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use derive_more as dm;
use std::mem::MaybeUninit;

pub enum CardinalDir {
    North,
    South,
    East,
    West,
}

// Derive macros from the standard library.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// `derive_more` macros.
#[derive(dm::Add, dm::Sub)]
pub struct Vec2 {
    pub x: i64,
    pub y: i64,
}

impl Vec2 {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn pointing(dir: CardinalDir) -> Self {
        match dir {
            CardinalDir::North => Self::new(0, -1),
            CardinalDir::South => Self::new(0, 1),
            CardinalDir::West => Self::new(-1, 0),
            CardinalDir::East => Self::new(1, 0),
        }
    }
}

pub trait ArrayConcatExt<T, const N: usize> {
    fn array_concat<const M: usize>(self, other: [T; M]) -> [T; N + M];
}

impl<T: Copy, const N: usize> ArrayConcatExt<T, N> for [T; N] {
    fn array_concat<const M: usize>(self, other: [T; M]) -> [T; N + M] {
        let mut result = MaybeUninit::uninit_array();
        // Initialize the first N elements using `split_array_mut`.
        MaybeUninit::write_slice(result.split_array_mut::<N>().0, &self);
        // Initialize the last M elements using `rsplit_array_mut`.
        MaybeUninit::write_slice(result.rsplit_array_mut::<M>().1, &other);
        // SAFETY: the length of the result array is N+M. We initialized the
        // first N elements and then the last M elements, so all the elements
        // have been initialized.
        unsafe { MaybeUninit::array_assume_init(result) }
    }
}

pub trait ArrayUnzipExt<T, U, V, const N: usize> {
    fn unzip_array(self, f: impl FnMut(T) -> (U, V)) -> ([U; N], [V; N]);
}

impl<T, U, V, const N: usize> ArrayUnzipExt<T, U, V, N> for [T; N] {
    fn unzip_array(self, f: impl FnMut(T) -> (U, V)) -> ([U; N], [V; N]) {
        let mut left: [_; N] = MaybeUninit::uninit_array();
        let mut right: [_; N] = MaybeUninit::uninit_array();
        let src_iter = self.map(f).into_iter();
        let dst_iter = left.iter_mut().zip(right.iter_mut());
        for (src, dst) in src_iter.zip(dst_iter) {
            dst.0.write(src.0);
            dst.1.write(src.1);
        }
        // SAFETY: TODO
        let left = unsafe { MaybeUninit::array_assume_init(left) };
        // SAFETY: TODO
        let right = unsafe { MaybeUninit::array_assume_init(right) };
        (left, right)
    }
}
