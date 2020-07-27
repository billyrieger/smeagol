// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::{bit::Bit8x8, grid::Grid2};
use crate::Result;
use crate::Error;
use crate::life::Cell;
use crate::life::Position;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Leaf {
    cells: Bit8x8,
}

impl Leaf {
    pub const LEVEL: u8 = 3;
    pub const SIDE_LEN: u8 = 8;

    pub fn empty() -> Self {
        Self {
            cells: Bit8x8::zeros(),
        }
    }

    fn pos_to_index(pos: Position) -> usize {
        let side_len = Self::SIDE_LEN as usize;
        let half_side_len = (side_len / 2) as i64;

        let col = (pos.x + half_side_len) as usize;
        let row = (pos.y + half_side_len) as usize;

        row * side_len + col
    }

    fn index_to_pos(index: u8) -> Position {
        let col = (index % Self::SIDE_LEN) as i64;
        let row = (index / Self::SIDE_LEN) as i64;
        Position::new(col - 4, row - 4)
    }

    pub fn get_cell(&self, pos: Position) -> Cell {
        let half_side_len = (Self::SIDE_LEN / 2) as i64;

        let index = Self::pos_to_index(pos);
        if self.cells.get_bit(index) {
            Cell::Alive
        } else {
            Cell::Dead
        }
    }
}

pub trait Rule {
    fn evolve(&mut self, quadleaf: Grid2<Leaf>, steps: u64) -> Leaf;
}

// pub struct B3S23<B>(std::marker::PhantomData<B>);

// impl<B> Rule for B3S23<B>
// where
//     B: BitSquare,
// {
//     type Leaf = B;

//     fn evolve(&mut self, _grid: Grid2<B>, steps: u64) -> B {
//         assert!(steps <= 4);

//         let half_adder = |sum: &mut B, addend: B| -> B {
//             let carry = *sum & addend;
//             *sum = *sum ^ addend;
//             carry
//         };

//         let step_once = |alive: B| -> B {
//             let mut sum: [B; 3] = [B::zero(); 3];
//             for &addend in &alive.moore_neighborhood() {
//                 let carry = half_adder(&mut sum[0], addend);
//                 let carry = half_adder(&mut sum[1], carry);
//                 sum[2] = sum[2] | carry;
//             }

//             // two is 010 is binary
//             let total_is_two = !sum[2] & sum[1] & !sum[0];

//             // three is 011 is binary
//             let total_is_three = !sum[2] & sum[1] & sum[0];

//             total_is_three | (alive & total_is_two)
//         };

//         let mut result = B::zero();
//         for _ in 0..steps {
//             result = step_once(result);
//         }
//         // Leaf::new(crate::util::center(result).into())
//         todo!()
//     }
// }
