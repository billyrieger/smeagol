#![feature(const_fn_trait_bound, portable_simd)]

use indexmap::IndexSet;
use std::simd::{u16x16, u8x8};

#[derive(Clone, Copy, Debug)]
pub struct Grid2<T>(pub [T; 4]);

impl<T> Grid2<T> {
    const fn nw(&self) -> &T {
        &self.0[0]
    }

    const fn ne(&self) -> &T {
        &self.0[1]
    }

    const fn sw(&self) -> &T {
        &self.0[2]
    }

    const fn se(&self) -> &T {
        &self.0[3]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Leaf(u8x8);

#[derive(Clone, Copy, Debug)]
pub struct Clover(u16x16);

impl Clover {
    pub const fn from_leaves(leaves: Grid2<Leaf>) -> Self {
        let [nw, ne, sw, se] = [
            leaves.nw().0.as_array(),
            leaves.ne().0.as_array(),
            leaves.sw().0.as_array(),
            leaves.se().0.as_array(),
        ];
        let rows = [
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
        ];
        Self(u16x16::from_array(rows))
    }

    pub const fn center(&self) -> Leaf {
        let [_, _, _, _, a, b, c, d, e, f, g, h, _, _, _, _] = *self.0.as_array();
        let rows = [
            (a >> 4) as u8,
            (b >> 4) as u8,
            (c >> 4) as u8,
            (d >> 4) as u8,
            (e >> 4) as u8,
            (f >> 4) as u8,
            (g >> 4) as u8,
            (h >> 4) as u8,
        ];
        Leaf(u8x8::from_array(rows))
    }
}

pub trait LifeRule {
    fn step(&self, cells: Clover) -> Clover;

    fn advance(&self, leaves: Grid2<Leaf>, steps: u8) -> Leaf {
        let mut clover = Clover::from_leaves(leaves);
        for _ in 0..(steps % 4) {
            clover = self.step(clover);
        }
        clover.center()
    }
}

pub struct B3S23;

impl LifeRule for B3S23 {
    fn step(&self, Clover(a): Clover) -> Clover {
        let (aw, ae) = (a << u16x16::splat(1), a >> u16x16::splat(1));
        let (s0, s1) = (aw ^ ae, aw & ae);
        let (hs0, hs1) = (s0 ^ a, (s0 & a) | s1);
        let (hs0w8, hs0e8) = (hs0.rotate_lanes_left::<1>(), hs0.rotate_lanes_right::<1>());
        let (hs1w8, hs1e8) = (hs1.rotate_lanes_left::<1>(), hs1.rotate_lanes_right::<1>());
        let ts0 = hs0w8 ^ hs0e8;
        let ts1 = (hs0w8 & hs0e8) | (ts0 & s0);
        Clover((hs1w8 ^ hs1e8 ^ ts1 ^ s1) & ((hs1w8 | hs1e8) ^ (ts1 | s1)) & ((ts0 ^ s0) | a))
    }
}

enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

struct NodeId(usize);

struct Branch {
    side_log_2: u8,
    children: Grid2<NodeId>,
}

struct Arena {
    nodes: IndexSet<Node>,
}

impl Arena {
    fn advance(&mut self, leaves: Grid2<Grid2<Leaf>>, steps: u8) {
        let steps = steps % 8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let empty = Leaf(u8x8::splat(0));
        let mut glider = [0u8; 8];
        glider[4] = 0b010;
        glider[5] = 0b001;
        glider[6] = 0b111;
        let glider = Leaf(u8x8::from_array(glider));
        let cells = Clover::from_leaves(Grid2([glider, empty, empty, empty])).0;
        println!("{cells:#018b}");
        let cells = B3S23.step(B3S23.step(Clover(cells))).0;
        println!("{cells:#018b}");
        let center = Clover(cells).center().0;
        println!("{center:#010b}");
    }
}
