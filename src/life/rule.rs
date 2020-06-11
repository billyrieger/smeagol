// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    life::quadtree::Leaf,
    util::{Bit16x16, Grid2},
};

pub trait Rule {
    type Leaf;

    fn evolve(&self, grid: Grid2<Self::Leaf>, steps: u8) -> Self::Leaf;
}

pub struct B3S23;

impl Rule for B3S23 {
    type Leaf = Leaf;

    fn evolve(&self, grid: Grid2<Leaf>, steps: u8) -> Leaf {
        type B = Bit16x16;
        assert!(steps <= 4);

        let half_adder = |sum: &mut B, addend: B| -> B {
            let carry = *sum & addend;
            *sum ^= addend;
            carry
        };

        let step_once = |alive: B| -> B {
            let mut sum: [B; 3] = Default::default();
            for &addend in &alive.moore_neighborhood() {
                let carry = half_adder(&mut sum[0], addend);
                let carry = half_adder(&mut sum[1], carry);
                sum[2] |= carry;
            }

            // two is 010 is binary
            let total_is_two = !sum[2] & sum[1] & !sum[0];

            // three is 011 is binary
            let total_is_three = !sum[2] & sum[1] & sum[0];

            total_is_three | (alive & total_is_two)
        };

        let mut result: Bit16x16 = crate::util::combine(grid.map(|leaf| {
            let nw: [u8; 8] = leaf.alive.0.to_be_bytes();
            todo!()
        }));
        for _ in 0..steps {
            result = step_once(result);
        }
        Leaf::new(crate::util::center(result).into())
    }
}
