// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use packed_simd::{shuffle, u16x16};

#[derive(Clone, Copy, Debug)]
pub struct Grid2<T>([T; 4]);

impl<T> Grid2<T>
where
    T: Copy,
{
    pub fn unpack(&self) -> [T; 4] {
        self.0
    }

    pub fn nw(&self) -> T {
        self.0[0]
    }
    pub fn ne(&self) -> T {
        self.0[1]
    }
    pub fn sw(&self) -> T {
        self.0[2]
    }
    pub fn se(&self) -> T {
        self.0[3]
    }
}

pub trait Rule {
    type Leaf;

    fn evolve(&self, grid: Grid2<Self::Leaf>, steps: u8) -> Self::Leaf;
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TwoStateLeaf {
    alive: [u8; 8],
}

fn grid_to_u16x16(grid: Grid2<TwoStateLeaf>) -> u16x16 {
    let [nw, ne, sw, se] = grid.unpack();
    let [nw, ne, sw, se] = [nw.alive, ne.alive, sw.alive, se.alive];

    u16x16::new(
        u16::from_be_bytes([nw[0], ne[0]]),
        u16::from_be_bytes([nw[1], ne[1]]),
        u16::from_be_bytes([nw[2], ne[2]]),
        u16::from_be_bytes([nw[3], ne[3]]),
        u16::from_be_bytes([nw[4], ne[4]]),
        u16::from_be_bytes([nw[5], ne[5]]),
        u16::from_be_bytes([nw[6], ne[6]]),
        u16::from_be_bytes([nw[7], ne[7]]),
        u16::from_be_bytes([sw[0], se[0]]),
        u16::from_be_bytes([sw[1], se[1]]),
        u16::from_be_bytes([sw[2], se[2]]),
        u16::from_be_bytes([sw[3], se[3]]),
        u16::from_be_bytes([sw[4], se[4]]),
        u16::from_be_bytes([sw[5], se[5]]),
        u16::from_be_bytes([sw[6], se[6]]),
        u16::from_be_bytes([sw[7], se[7]]),
    )
}

fn center(grid: u16x16) -> TwoStateLeaf {
    let grid: [u16; 16] = grid.into();

    let middle = |row: u16| (row >> 4) as u8;

    TwoStateLeaf {
        alive: [
            middle(grid[4]),
            middle(grid[5]),
            middle(grid[6]),
            middle(grid[7]),
            middle(grid[8]),
            middle(grid[9]),
            middle(grid[10]),
            middle(grid[11]),
        ],
    }
}

fn rotate_up(grid: u16x16) -> u16x16 {
    shuffle!(grid, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0])
}

fn rotate_down(grid: u16x16) -> u16x16 {
    shuffle!(grid, [15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14])
}

fn rotate_left(grid: u16x16) -> u16x16 {
    grid.rotate_left(u16x16::splat(1))
}

fn rotate_right(grid: u16x16) -> u16x16 {
    grid.rotate_right(u16x16::splat(1))
}

fn half_adder(sum: &mut u16x16, addend: u16x16) -> u16x16 {
    let carry = *sum & addend;
    *sum ^= addend;
    carry
}

pub struct B3S23;

impl Rule for B3S23 {
    type Leaf = TwoStateLeaf;

    fn evolve(&self, grid: Grid2<TwoStateLeaf>, steps: u8) -> TwoStateLeaf {
        assert!(steps <= 4);

        let step_once = |alive: u16x16| -> u16x16 {
            let [mut d2, mut d1, mut d0] = [u16x16::splat(0); 3];

            let moore_neighborhood = [
                rotate_up(alive),
                rotate_down(alive),
                rotate_left(alive),
                rotate_right(alive),
                rotate_up(rotate_left(alive)),
                rotate_up(rotate_right(alive)),
                rotate_down(rotate_left(alive)),
                rotate_down(rotate_right(alive)),
            ];

            for &addend in &moore_neighborhood {
                let carry0 = half_adder(&mut d0, addend);
                let carry1 = half_adder(&mut d1, carry0);
                d2 |= carry1;
            }

            // two is 010 is binary
            let sum_is_two = !d2 & d1 & !d0;

            // three is 011 in binary
            let sum_is_three = !d2 & d1 & d0;

            sum_is_three | (alive & sum_is_two)
        };

        let mut result: u16x16 = grid_to_u16x16(grid);
        for _ in 0..steps {
            result = step_once(result);
        }
        center(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let rule = B3S23;

        let nw = TwoStateLeaf {
            alive: [
                0b_00000000,
                0b_00000000,
                0b_00000000,
                0b_00000000,
                0b_00000100,
                0b_00000010,
                0b_00001110,
                0b_00000000,
            ],
        };
        let ne = TwoStateLeaf { alive: [0; 8] };
        let sw = TwoStateLeaf { alive: [0; 8] };
        let se = TwoStateLeaf { alive: [0; 8] };

        let grid = Grid2([nw, ne, sw, se]);

        let expected0 = TwoStateLeaf {
            alive: [
                0b_01000000,
                0b_00100000,
                0b_11100000,
                0b_00000000,
                0b_00000000,
                0b_00000000,
                0b_00000000,
                0b_00000000,
            ],
        };

        let expected4 = TwoStateLeaf {
            alive: [
                0b_00000000,
                0b_00100000,
                0b_00010000,
                0b_01110000,
                0b_00000000,
                0b_00000000,
                0b_00000000,
                0b_00000000,
            ],
        };

        assert_eq!(rule.evolve(grid, 0), expected0);
        assert_eq!(rule.evolve(grid, 4), expected4);
    }
}
