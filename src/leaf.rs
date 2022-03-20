// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::{ArrayConcatExt, ArrayUnzipExt, BitGrid, Dir, Grid2, ToGrid};

use std::simd::{u16x16, u8x8};

use derive_more as dm;

// Derive macros from the standard library.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// `derive_more` macros.
#[derive(dm::BitAnd, dm::BitOr, dm::BitXor, dm::Not)]
pub struct Leaf {
    pub cells: u16x16,
}

impl Leaf {
    pub const SIDE: u8 = 16;
    pub const SIDE_LOG2: u8 = 4;

    pub const fn new(cells: u16x16) -> Self {
        Self { cells }
    }

    pub const fn empty() -> Self {
        Self::new(u16x16::splat(0))
    }

    pub fn from_parts(Grid2 { nw, ne, sw, se }: Grid2<QuarterLeaf>) -> Self {
        let west = nw.cells.to_array().array_concat(sw.cells.to_array());
        let east = ne.cells.to_array().array_concat(se.cells.to_array());
        let whole = west.zip(east).map(|(w, e)| u16::from_be_bytes([w, e]));
        Self::new(u16x16::from_array(whole))
    }

    pub fn to_parts(self) -> Grid2<QuarterLeaf> {
        let (west, east) = self
            .cells
            .to_array()
            .map(|row| row.to_be_bytes())
            .map(|[w, e]| (w, e))
            .unzip_array();
        let (nw, ne) = (west.split_array_ref().0, east.split_array_ref().0);
        let (sw, se) = (west.rsplit_array_ref().1, east.rsplit_array_ref().1);
        [*nw, *ne, *sw, *se]
            .map(u8x8::from_array)
            .map(QuarterLeaf::new)
            .to_grid()
    }

    pub fn population(&self) -> u128 {
        self.cells
            .to_array()
            .map(|row| u128::from(row.count_ones()))
            .iter()
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::empty()
    }

    pub fn center(&self) -> QuarterLeaf {
        // Start with 16 rows and 16 columns.
        let rows = self.cells.as_array();
        // Keep the first 12 of the 16 rows, thereby removing the last 4 rows.
        let rows: &[u16; 12] = rows.split_array_ref().0;
        // Then keep the last 8 of those 12 rows, thereby removing the first 4 rows.
        let rows: &[u16; 8] = rows.rsplit_array_ref().1;
        // Shift each row to the right by 4 and keep the right 8 columns.
        let rows: [u8; 8] = rows.map(|row| (row >> 4) as u8);
        // The final `u8x8` is the center of the original `u16x16`.
        QuarterLeaf::new(u8x8::from_array(rows))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// `derive_more` macros.
#[derive(dm::BitAnd, dm::BitOr, dm::BitXor, dm::Not)]
pub struct QuarterLeaf {
    cells: u8x8,
}

impl QuarterLeaf {
    fn new(cells: u8x8) -> Self {
        Self { cells }
    }
}

trait CoreRule {
    fn step_once<B: BitGrid>(&mut self, grid: B) -> B;
}

struct Conway;

impl CoreRule for Conway {
    fn step_once<B: BitGrid>(&mut self, a: B) -> B {
        // TODO: figure out how this works.
        // Original algorithm:
        //     Rokicki, Tomas. “Life Algorithms,” June 28, 2018.
        let (aw, ae) = (a.shift(Dir::West), a.shift(Dir::East));
        let (s0, s1) = (aw ^ ae, aw & ae);
        let (hs0, hs1) = (s0 ^ a, (s0 & a) | s1);
        let (hs0w8, hs0e8) = (hs0.shift(Dir::North), hs0.shift(Dir::South));
        let (hs1w8, hs1e8) = (hs1.shift(Dir::North), hs1.shift(Dir::South));
        let ts0 = hs0w8 ^ hs0e8;
        let ts1 = (hs0w8 & hs0e8) | (ts0 & s0);
        (hs1w8 ^ hs1e8 ^ ts1 ^ s1) & ((hs1w8 | hs1e8) ^ (ts1 | s1)) & ((ts0 ^ s0) | a)
    }
}

#[test]
fn test() {
    let mut rows = [0; 8];
    rows[2] = 0b00100;
    rows[3] = 0b01000;
    rows[4] = 0b01110;
    let x = u8x8::from_array(rows);
    dbg!(x);
    dbg!(Conway.step_once(Conway.step_once(Conway.step_once(Conway.step_once(x)))));
}
