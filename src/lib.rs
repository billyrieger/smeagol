#![feature(const_fn_trait_bound, portable_simd)]

use std::simd::{u16x16, u16x8, u8x8};

#[derive(Clone, Copy, Debug)]
struct Leaf(u8x8);

#[derive(Clone, Copy, Debug)]
struct Clover(u16x16);

impl Clover {
    fn from_leaves(leaves: Grid2<Leaf>) -> Self {
        let mut rows = [0u16; 16];
        let (north, south) = rows.split_at_mut(8);
        let extend = |leaf: Leaf| u16x8::from_array(leaf.0.to_array().map(u16::from));
        let combine = |west: Leaf, east: Leaf| (extend(west) << u16x8::splat(8)) | extend(east);
        north.copy_from_slice(combine(leaves.nw, leaves.ne).as_ref());
        south.copy_from_slice(combine(leaves.sw, leaves.se).as_ref());
        Self(u16x16::from_array(rows))
    }
}

#[derive(Clone, Copy, Debug)]
struct Grid2<T> {
    nw: T,
    ne: T,
    sw: T,
    se: T,
}

impl<T> Grid2<T>
where
    T: Copy,
{
    const fn from_array([nw, ne, sw, se]: [T; 4]) -> Self {
        Self { nw, ne, sw, se }
    }

    const fn as_array(&self) -> [T; 4] {
        [self.nw, self.ne, self.sw, self.se]
    }
}

trait LifeRule {
    fn step(&self, cells: Clover, steps: u32) -> Clover;
}

struct B3S23;

impl LifeRule for B3S23 {
    fn step(&self, Clover(mut a): Clover, steps: u32) -> Clover {
        for _ in 0..steps {
            let (aw, ae) = (a << u16x16::splat(1), a >> u16x16::splat(1));
            let (s0, s1) = (aw ^ ae, aw & ae);
            let (hs0, hs1) = (s0 ^ a, (s0 & a) | s1);
            let (hs0w8, hs0e8) = (hs0.rotate_lanes_left::<1>(), hs0.rotate_lanes_right::<1>());
            let (hs1w8, hs1e8) = (hs1.rotate_lanes_left::<1>(), hs1.rotate_lanes_right::<1>());
            let ts0 = hs0w8 ^ hs0e8;
            let ts1 = (hs0w8 & hs0e8) | (ts0 & s0);
            a = (hs1w8 ^ hs1e8 ^ ts1 ^ s1) & ((hs1w8 | hs1e8) ^ (ts1 | s1)) & ((ts0 ^ s0) | a);
        }
        Clover(a)
    }
}

enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

struct NodeId(usize);

struct Branch {
    side_len_log2: u8,
    children: Grid2<NodeId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut cells = u16x16::splat(0);
        let rows = cells.as_mut_array();
        rows[4] = 0b010 << 5;
        rows[5] = 0b001 << 5;
        rows[6] = 0b111 << 5;
        println!("{cells:#016b}");
        let cells = B3S23.step(Clover(cells), 4).0;
        println!("{cells:#016b}");
    }
}
