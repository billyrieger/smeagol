// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::grid::Grid2;

use std::hash::Hash;

pub trait Leaf: Copy + Default + Eq + Hash {
    const SIDE_LEN: u32;
    const LOG_SIDE_LEN: u8;

    type Cell;
}

// impl<B> Leaf for B
// where
//     B: BitSquare,
// {
//     const SIDE_LEN: u32 = B::SIDE_LEN;
//     const LOG_SIDE_LEN: u8 = B::LOG_SIDE_LEN;
//     type Cell = bool;
// }

pub trait Rule {
    type Leaf: Leaf;

    fn evolve(&mut self, grid: Grid2<Self::Leaf>, steps: u64) -> Self::Leaf;
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
