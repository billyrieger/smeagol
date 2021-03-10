// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;

use derive_more::{BitAnd, BitOr, BitXor, Not, Shl, Shr};
use packed_simd::{shuffle, u16x16, u8x8};

use crate::util::Grid2;

pub trait Rule {
    fn step(&self, cells: Clover) -> Clover;
}

#[derive(Clone, Copy, Debug)]
pub struct B3S23;

impl Rule for B3S23 {
    fn step(&self, a: Clover) -> Clover {
        // Adapted from Tomas Rokicki's "Life Algorithms."
        // https://www.gathering4gardner.org/g4g13gift/math/RokickiTomas-GiftExchange-LifeAlgorithms-G4G13.pdf
        let (aw, ae) = (a << 1, a >> 1);
        let (s0, s1) = (aw ^ ae, aw & ae);
        let (hs0, hs1) = (s0 ^ a, (s0 & a) | s1);
        let (hs0w8, hs0e8) = (hs0.shift_up(), hs0.shift_down());
        let (hs1w8, hs1e8) = (hs1.shift_up(), hs1.shift_down());
        let ts0 = hs0w8 ^ hs0e8;
        let ts1 = (hs0w8 & hs0e8) | (ts0 & s0);
        (hs1w8 ^ hs1e8 ^ ts1 ^ s1) & ((hs1w8 | hs1e8) ^ (ts1 | s1)) & ((ts0 ^ s0) | a)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
// `derive_more` macros
#[derive(BitAnd, BitOr, BitXor, Not, Shl, Shr)]
pub struct Leaf {
    cells: u8x8,
}

impl fmt::Display for Leaf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rows: [u8; 8] = self.cells.into();
        for row in &rows {
            writeln!(f, "{:08b}", row)?;
        }
        Ok(())
    }
}


impl Leaf {
    pub const fn new(cells: u8x8) -> Self {
        Self { cells }
    }

    pub const fn level() -> usize {
        3
    }

    pub const fn min_coord() -> i64 {
        -4
    }

    pub const fn max_coord() -> i64 {
        3
    }

    pub const fn is_inbounds(pos: crate::Coords) -> bool {
        Self::min_coord() <= pos.x
            && Self::min_coord() <= pos.y
            && pos.x <= Self::max_coord()
            && pos.y <= Self::max_coord()
    }

    pub fn empty() -> Self {
        Self {
            cells: u8x8::splat(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cells == u8x8::splat(0)
    }

    pub fn set_cell(&self, pos: crate::Coords) -> Self {
        assert!(Self::is_inbounds(pos));
        let row = (pos.y + 4) as usize;
        let col = (3 - pos.x) as usize;
        unsafe {
            Self {
                cells: self
                    .cells
                    .replace_unchecked(row, self.cells.extract_unchecked(row) | (1 << col)),
            }
        }
    }

    pub fn unset_cell(&self, pos: crate::Coords) -> Self {
        assert!(Self::is_inbounds(pos));
        let row = (pos.y + 4) as usize;
        let col = (pos.x + 4) as usize;
        unsafe {
            Self {
                cells: self
                    .cells
                    .replace_unchecked(row, self.cells.extract_unchecked(row) & !(1 << col)),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
// `derive_more` macros
#[derive(BitAnd, BitOr, BitXor, Not, Shl, Shr)]
pub struct Clover {
    cells: u16x16,
}

impl Clover {
    pub fn new(leaves: Grid2<Leaf>) -> Self {
        let nw: [u8; 8] = leaves.nw.cells.into();
        let ne: [u8; 8] = leaves.ne.cells.into();
        let sw: [u8; 8] = leaves.sw.cells.into();
        let se: [u8; 8] = leaves.se.cells.into();

        let combine = |west: u8, east: u8| -> u16 { ((west as u16) << 8) | (east as u16) };

        let cells: u16x16 = [
            combine(nw[0], ne[0]),
            combine(nw[1], ne[1]),
            combine(nw[2], ne[2]),
            combine(nw[3], ne[3]),
            combine(nw[4], ne[4]),
            combine(nw[5], ne[5]),
            combine(nw[6], ne[6]),
            combine(nw[7], ne[7]),
            combine(sw[0], se[0]),
            combine(sw[1], se[1]),
            combine(sw[2], se[2]),
            combine(sw[3], se[3]),
            combine(sw[4], se[4]),
            combine(sw[5], se[5]),
            combine(sw[6], se[6]),
            combine(sw[7], se[7]),
        ]
        .into();

        Self { cells }
    }

    pub fn center(&self) -> Leaf {
        let rows: [u16; 16] = self.cells.into();
        let mut center = [0u8; 8];

        for i in 0..8 {
            center[i] = (rows[i + 4] >> 4) as u8;
        }

        Leaf::new(center.into())
    }

    pub fn shift_down(&self) -> Self {
        let cells = shuffle!(
            self.cells,
            [15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        );
        Self { cells }
    }

    pub fn shift_up(&self) -> Self {
        let cells = shuffle!(
            self.cells,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0]
        );
        Self { cells }
    }
}

impl fmt::Display for Clover {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rows: [u16; 16] = self.cells.into();
        for row in &rows {
            writeln!(f, "{:016b}", row)?;
        }
        Ok(())
    }
}
