pub mod mem;

use std::{fmt, hash::Hash};

use derive_more::{BitAnd, BitOr, BitXor, Not, Shl, Shr};
use packed_simd::{shuffle, u16x16, u8x8};

pub trait Rule {
    fn step(&self, cells: Clover) -> Clover;
}

pub struct B3S23;

impl Rule for B3S23 {
    fn step(&self, a: Clover) -> Clover {
        // Adapted from the `gen3` function of Tomas Rokicki's "Life Algorithms."
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
// `derive_more` derive macros
#[derive(BitAnd, BitOr, BitXor, Not, Shl, Shr)]
pub struct Leaf {
    cells: u8x8,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
// `derive_more` derive macros
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

pub struct Branch {
    _nw: usize,
    _ne: usize,
    _sw: usize,
    _se: usize,
}

pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

// pub struct Universe {
//     nodes: mem::Arena<Node>,
//     root: Node,
// }
