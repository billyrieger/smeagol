// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::life::LifeRule;
use crate::util::{ArrayConcatExt, ArrayUnzipExt, BitGrid, Grid2, ToGrid};

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

    pub fn from_parts(Grid2 { nw, ne, sw, se }: Grid2<LeafPart>) -> Self {
        let west = nw.cells.to_array().array_concat(sw.cells.to_array());
        let east = ne.cells.to_array().array_concat(se.cells.to_array());
        let whole = west.zip(east).map(|(w, e)| u16::from_be_bytes([w, e]));
        Self::new(u16x16::from_array(whole))
    }

    pub fn to_parts(self) -> Grid2<LeafPart> {
        let (west, east) = self
            .cells
            .to_array()
            .map(|row| row.to_be_bytes())
            .unzip_array(|[w, e]| (w, e));
        let (nw, ne) = (west.split_array_ref().0, east.split_array_ref().0);
        let (sw, se) = (west.rsplit_array_ref().1, east.rsplit_array_ref().1);
        [*nw, *ne, *sw, *se]
            .map(|part| LeafPart::new(u8x8::from_array(part)))
            .to_grid()
    }

    pub fn step<R: LifeRule>(&self, rule: &R, ticks: u8) -> Self {
        let mut cells = self.cells;
        for _ in 0..ticks {
            cells = rule.tick(cells);
        }
        Self::new(cells)
    }

    pub fn population(&self) -> u128 {
        u128::from(self.cells.count_ones())
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::empty()
    }

    pub fn center(&self) -> LeafPart {
        // Start with 16 rows and 16 columns. To isolate the central 8 rows,
        // split off the first 12 rows and then split off the last 8 of those 12
        // rows. To isolate the central 8 columns, shift each row to the right
        // by 4 and keep the right 8 columns.  The final `u8x8` is the center of
        // the original `u16x16`.
        let rows: &[u16; 16] = self.cells.as_array();
        let rows: &[u16; 12] = rows.split_array_ref().0;
        let rows: &[u16; 8] = rows.rsplit_array_ref().1;
        let rows: [u8; 8] = rows.map(|row| (row >> 4) as u8);
        LeafPart::new(u8x8::from_array(rows))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// `derive_more` macros.
#[derive(dm::BitAnd, dm::BitOr, dm::BitXor, dm::Not)]
pub struct LeafPart {
    cells: u8x8,
}

impl LeafPart {
    fn new(cells: u8x8) -> Self {
        Self { cells }
    }
}
