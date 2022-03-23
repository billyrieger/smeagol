use std::mem::size_of;
use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};

use num::PrimInt;

struct Quad<T> {
    nw: T,
    ne: T,
    sw: T,
    se: T,
}

impl<T> Quad<T> {
    fn to_array(self) -> [T; 4] {
        [self.nw, self.ne, self.sw, self.se]
    }

    fn from_array([nw, ne, sw, se]: [T; 4]) -> Self {
        Self { nw, ne, sw, se }
    }

    fn map<U>(self, f: impl FnMut(T) -> U) -> Quad<U> {
        self.to_array().map(f).to_quad()
    }

    fn try_map<U>(self, f: impl FnMut(T) -> Option<U>) -> Option<Quad<U>> {
        Some(self.to_array().try_map(f)?.to_quad())
    }
}

trait ArrayToQuadExt<T> {
    fn to_quad(self) -> Quad<T>;
}

impl<T> ArrayToQuadExt<T> for [T; 4] {
    fn to_quad(self) -> Quad<T> {
        Quad::from_array(self)
    }
}

trait SimdExt<T, const LANES: usize>: Sized
where
    T: SimdElement,
    LaneCount<LANES>: SupportedLaneCount,
{
    const ROWS: usize;
    const COLS: usize;

    fn get(&self, row: usize, col: usize) -> Option<bool>;
    fn set(&self, row: usize, col: usize, value: bool) -> Option<Self>;
}

impl<T, const LANES: usize> SimdExt<T, LANES> for Simd<T, LANES>
where
    T: SimdElement + PrimInt,
    LaneCount<LANES>: SupportedLaneCount,
{
    const ROWS: usize = LANES;
    const COLS: usize = size_of::<T>() * 8;

    fn get(&self, row: usize, col: usize) -> Option<bool> {
        (row < Self::ROWS && col < Self::COLS).then(|| self[row] & (T::one() << col) > T::zero())
    }

    fn set(&self, row: usize, col: usize, value: bool) -> Option<Self> {
        (row < LANES && col < size_of::<T>() * 8).then(|| {
            let mut result = *self;
            result[row] = if value {
                result[row] | (T::one() << col)
            } else {
                result[row] & !(T::one() << col)
            };
            result
        })
    }
}

#[test]
fn test() {
    use std::simd::u16x16;
    let mut x = u16x16::splat(0);
    x[4] = 201;
    x[11] = 230;
    println!("{:#016b}", x);
}

trait ToParts {
    type Part;

    fn split(self) -> Quad<Self::Part>;
    fn combine(parts: Quad<Self::Part>) -> Self;
}
