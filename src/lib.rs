#![feature(iter_partition_in_place)]
#![allow(dead_code)]

use std::fmt;
use std::hash::Hash;

use derive_more::{BitAnd, BitOr, BitXor, Not, Shl, Shr};
use indexmap::{indexset, IndexSet};
use packed_simd::{shuffle, u16x16, u8x8};

pub trait Rule {
    fn step(&self, cells: Clover) -> Clover;
}

pub struct B3S23;

impl Rule for B3S23 {
    fn step(&self, a: Clover) -> Clover {
        // Adapted from Tomas Rokicki's "Life Algorithms." Don't ask me how it works.
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

impl Leaf {
    #[inline(always)]
    const fn level() -> usize {
        3
    }

    #[inline(always)]
    const fn min_coord() -> i64 {
        -4
    }

    #[inline(always)]
    const fn max_coord() -> i64 {
        3
    }

    #[inline(always)]
    const fn is_inbounds(pos: Coords) -> bool {
        Self::min_coord() <= pos.x
            && Self::min_coord() <= pos.y
            && pos.x <= Self::max_coord()
            && pos.y <= Self::max_coord()
    }

    fn empty() -> Self {
        Self {
            cells: u8x8::splat(0),
        }
    }

    fn set_cell(&self, pos: Coords) -> Self {
        debug_assert!(Self::is_inbounds(pos));
        let row = (pos.y + 4) as usize;
        let col = (pos.x + 4) as usize;
        unsafe {
            Self {
                cells: self
                    .cells
                    .replace_unchecked(row, self.cells.extract_unchecked(row) | (1 << col)),
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Coords {
    x: i64,
    y: i64,
}

impl Coords {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    fn offset(&self, dx: i64, dy: i64) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    fn relative_to(&self, other: Self) -> Self {
        self.offset(-other.x, -other.y)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Branch {
    nw: usize,
    ne: usize,
    sw: usize,
    se: usize,
    level: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id {
    data: u64,
}

#[derive(Clone, Debug)]
pub struct Universe {
    base: IndexSet<Leaf>,
    tiers: Vec<IndexSet<Branch>>,
}

const MAX_LEVEL: usize = 63;
const MIN_COORD: i64 = -(1 << (MAX_LEVEL - 1));
const MAX_COORD: i64 = (1 << (MAX_LEVEL - 1)) - 1;

impl Universe {
    fn new() -> Self {
        let base = indexset! { Leaf::empty() };
        let mut tiers: Vec<IndexSet<Branch>> = vec![];

        for level in 4..=MAX_LEVEL {
            let empty = Branch {
                level,
                nw: 0,
                ne: 0,
                sw: 0,
                se: 0,
            };
            tiers.push(indexset! { empty });
        }
        Self { base, tiers }
    }

    fn create_leaf(&mut self, leaf: Leaf) -> usize {
        self.base.insert_full(leaf).0
    }

    fn create_branch(&mut self, branch: Branch) -> usize {
        self.tiers[branch.level - Leaf::level() - 1]
            .insert_full(branch)
            .0
    }

    fn get_leaf(&self, index: usize) -> Leaf {
        *self.base.get_index(index).expect("invalid index")
    }

    fn get_branch(&self, index: usize, level: usize) -> Branch {
        *self.tiers[level - Leaf::level() - 1]
            .get_index(index)
            .expect("invalid index")
    }

    fn get_child_leaves(&self, branch: Branch) -> [Leaf; 4] {
        let nw = self.get_leaf(branch.nw);
        let ne = self.get_leaf(branch.ne);
        let sw = self.get_leaf(branch.sw);
        let se = self.get_leaf(branch.se);
        [nw, ne, sw, se]
    }

    fn set_leaf_cells(&mut self, mut leaf: Leaf, center: Coords, coords: &[Coords]) -> usize {
        for pos in coords {
            leaf = leaf.set_cell(pos.relative_to(center));
        }
        self.create_leaf(leaf)
    }

    fn set_cells(
        &mut self,
        index: usize,
        level: usize,
        center: Coords,
        coords: &mut [Coords],
    ) -> usize {
        if coords.is_empty() {
            return index;
        }

        if level == Leaf::level() {
            let leaf = self.get_leaf(index);
            self.set_leaf_cells(leaf, center, coords)
        } else {
            let branch = self.get_branch(index, level);
            let delta: i64 = 1 << (branch.level - 2);
            let nw_center = center.offset(-delta, -delta);
            let ne_center = center.offset(delta, -delta);
            let sw_center = center.offset(-delta, delta);
            let se_center = center.offset(delta, delta);

            let (north, south) = partition(coords, |pos| pos.y < center.y);
            let (nw_coords, ne_coords) = partition(north, |pos| pos.x < center.x);
            let (sw_coords, se_coords) = partition(south, |pos| pos.x < center.x);

            let nw = self.set_cells(branch.nw, level - 1, nw_center, nw_coords);
            let ne = self.set_cells(branch.ne, level - 1, ne_center, ne_coords);
            let sw = self.set_cells(branch.sw, level - 1, sw_center, sw_coords);
            let se = self.set_cells(branch.se, level - 1, se_center, se_coords);
            let branch = Branch {
                level: branch.level,
                nw,
                ne,
                sw,
                se,
            };
            self.create_branch(branch)
        }
    }
}

fn partition<F>(coords: &mut [Coords], predicate: F) -> (&mut [Coords], &mut [Coords])
where
    F: FnMut(&Coords) -> bool,
{
    let split = coords.iter_mut().partition_in_place(predicate);
    coords.split_at_mut(split)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut universe = Universe::new();
        let mut coords = vec![];
        for x in 0..8 {
            for y in 0..8 {
                coords.push(Coords::new(x, y));
            }
        }
        let new = universe.set_cells(0, MAX_LEVEL, Coords::new(0, 0), &mut coords);
        dbg!(&universe);
        dbg!(new);
        dbg!(std::mem::size_of::<Branch>());
    }
}