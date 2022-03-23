use std::ops::{BitXor, Shl, Shr};
use std::simd::{u16x16, LaneCount, Simd, SimdElement, SupportedLaneCount};

use num::PrimInt;

use crate::util::CardinalDir;

pub trait Center {
    type Part;

    fn center(&self) -> Self::Part;
}

pub trait BitHalfOp {
    type Output;

    fn center(self) -> Self::Output;
}

type BitHalf<T> = <T as BitHalfOp>::Output;

impl<T, const LANES: usize> Center for Simd<T, LANES>
where
    T: SimdElement + PrimInt + BitHalfOp,
    BitHalf<T>: SimdElement + PrimInt,
    LaneCount<LANES>: SupportedLaneCount,
    LaneCount<{ LANES / 2 }>: SupportedLaneCount,
    [(); 3 * LANES / 4]:,
{
    type Part = Simd<BitHalf<T>, { LANES / 2 }>;

    fn center(&self) -> Self::Part {
        let grid: [T; LANES] = self.to_array();
        let grid: &[T; 3 * LANES / 4] = grid.split_array_ref().0;
        let grid: &[T; LANES / 2] = grid.rsplit_array_ref().1;
        let grid: [BitHalf<T>; LANES / 2] = grid.map(|t| t.center());
        Simd::from_array(grid)
    }
}

pub trait BitGrid: Sized + Copy + Eq {
    const ROWS: usize;
    const COLS: usize;

    fn get(&self, row: usize, col: usize) -> Option<bool>;
    fn set(&self, row: usize, col: usize) -> Option<Self>;
    fn toggle(&self, row: usize, col: usize) -> Option<Self>;
    fn shift(&self, dir: CardinalDir) -> Self;
}

impl<T, const LANES: usize> BitGrid for Simd<T, LANES>
where
    Self: Shl<Output = Self> + Shr<Output = Self> + BitXor<Output = Self>,
    T: SimdElement + PrimInt,
    LaneCount<LANES>: SupportedLaneCount,
{
    const ROWS: usize = LANES;
    const COLS: usize = std::mem::size_of::<T>() * 8;

    fn get(&self, row: usize, col: usize) -> Option<bool> {
        (row < Self::ROWS && col < Self::COLS).then(|| {
            let bitmask = T::one() << (Self::COLS - col - 1);
            self[row] & bitmask > T::zero()
        })
    }

    fn set(&self, row: usize, col: usize) -> Option<Self> {
        (row < Self::ROWS && col < Self::COLS).then(|| {
            let mut result = *self;
            let bitmask = T::one() << (Self::COLS - col - 1);
            result[row] = result[row] | bitmask;
            result
        })
    }

    fn toggle(&self, row: usize, col: usize) -> Option<Self> {
        (row < Self::ROWS && col < Self::COLS).then(|| {
            let mut result = *self;
            let bitmask = T::one() << (Self::COLS - col - 1);
            result[row] = result[row] ^ bitmask;
            result
        })
    }

    fn shift(&self, dir: CardinalDir) -> Self {
        match dir {
            CardinalDir::North => self.rotate_lanes_left::<1>(),
            CardinalDir::South => self.rotate_lanes_right::<1>(),
            CardinalDir::East => self >> Self::splat(T::one()),
            CardinalDir::West => self << Self::splat(T::one()),
        }
    }
}

#[test]
fn test() {
    use std::simd::*;
    let _ = <u8x8 as BitGrid>::ROWS;
    let _ = <u16x16 as BitGrid>::ROWS;
}

pub fn foo(input: u16x16, row: usize, col: usize) -> Option<u16x16> {
    input.toggle(row, col)
}

#[test]
fn foo_test() {
    // dbg!(foo(u16x16::splat(1), 15, 15));
}
