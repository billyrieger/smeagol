// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{grid::Grid2, Bool8x8, Rule};
use slotmap::new_key_type;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(u8);

impl Level {
    const MAX_LEVEL: Self = Self(63);

    pub fn increment(self) -> Option<Self> {
        if self < Self::MAX_LEVEL {
            Some(Self(self.0 + 1))
        } else {
            None
        }
    }
}

new_key_type! {
    pub struct NodeId;
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

impl Node {
    pub fn level(&self) -> Level {
        match self {
            Self::Leaf(_) => Level(3),
            Self::Branch(branch) => branch.level,
        }
    }

    /// Returns the number of alive cells in the `Node`.
    pub fn population(&self) -> u128 {
        match self {
            Self::Leaf(leaf) => leaf.alive.0.count_ones() as u128,
            Self::Branch(branch) => branch.population,
        }
    }
}

/// An 8 by 8 grid of dead or alive cells in a cellular automaton.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Leaf {
    alive: Bool8x8,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Branch {
    pub children: Grid2<NodeId>,
    pub level: Level,
    pub population: u128,
}

impl Leaf {
    /// Creates a new `Leaf`.
    pub fn new(alive: Bool8x8) -> Self {
        Self { alive }
    }

    pub fn alive(&self) -> Bool8x8 {
        self.alive
    }

    pub fn step(&self, rule: Rule) -> Self {
        let (alive, dead) = (self.alive, !self.alive);

        let alive_neighbor_count = Bool8x8::sum(&[
            alive.up(1),
            alive.down(1),
            alive.left(1),
            alive.right(1),
            alive.up(1).left(1),
            alive.up(1).right(1),
            alive.down(1).left(1),
            alive.left(1).right(1),
        ]);

        let combine = |a: [Bool8x8; 9], b: [Bool8x8; 9]| -> Bool8x8 {
            a.iter()
                .zip(b.iter())
                .map(|(&a, &b)| a & b)
                .fold(Bool8x8::FALSE, |a, b| a | b)
        };

        let born = combine(alive_neighbor_count, rule.birth);
        let survives = combine(alive_neighbor_count, rule.survival);

        Self::new(dead & born | alive & survives)
    }
}

impl Grid2<Leaf> {
    pub fn jump(&self, rule: Rule) -> Leaf {
        let a = self.0[0].step(rule);
        let b = self.north().step(rule);
        let c = self.0[1].step(rule);
        let d = self.west().step(rule);
        let e = self.center().step(rule);
        let f = self.east().step(rule);
        let g = self.0[2].step(rule);
        let h = self.south().step(rule);
        let i = self.0[3].step(rule);

        let mask_center = Bool8x8(0x0000_3C3C_3C3C_0000);
        let combine_jumps = |nw: Leaf, ne: Leaf, sw: Leaf, se: Leaf| {
            Leaf::new(
                Bool8x8::FALSE
                    | (nw.alive & mask_center).up(2).left(2)
                    | (ne.alive & mask_center).up(2).right(2)
                    | (sw.alive & mask_center).down(2).left(2)
                    | (se.alive & mask_center).down(2).right(2),
            )
        };

        let w = combine_jumps(a, b, d, e).step(rule);
        let x = combine_jumps(b, c, e, f).step(rule);
        let y = combine_jumps(d, e, g, h).step(rule);
        let z = combine_jumps(e, f, h, i).step(rule);

        combine_jumps(w, x, y, z)
    }

    fn join_horizontal(left: Leaf, right: Leaf) -> Leaf {
        let mask_left = Bool8x8(0xFF00_FF00_FF00_FF00);
        let mask_right = Bool8x8(0x00FF00_00FF_00FF_00FF);
        Leaf::new(
            Bool8x8::FALSE | left.alive.left(4) & mask_left | right.alive.right(4) & mask_right,
        )
    }

    fn join_vertical(top: Leaf, bottom: Leaf) -> Leaf {
        let mask_top = Bool8x8(0xFFFF_FFFF_0000_0000);
        let mask_bottom = Bool8x8(0x0000_0000_FFFF_FFFF);
        Leaf::new(Bool8x8::FALSE | top.alive.up(4) & mask_top | bottom.alive.down(4) & mask_bottom)
    }

    fn north(&self) -> Leaf {
        Self::join_horizontal(self.0[0], self.0[1])
    }

    fn south(&self) -> Leaf {
        Self::join_horizontal(self.0[2], self.0[3])
    }

    fn east(&self) -> Leaf {
        Self::join_vertical(self.0[0], self.0[2])
    }

    fn west(&self) -> Leaf {
        Self::join_vertical(self.0[1], self.0[3])
    }

    fn center(&self) -> Leaf {
        let mask_nw = Bool8x8(0xF0F0_F0F0_0000_0000);
        let mask_ne = Bool8x8(0x0F0F_0F0F_0000_0000);
        let mask_sw = Bool8x8(0x0000_0000_F0F0_F0F0);
        let mask_se = Bool8x8(0x0000_0000_0F0F_0F0F);

        let center = Bool8x8::FALSE
            | self.0[0].alive.up(4).left(4) & mask_nw
            | self.0[1].alive.up(4).right(4) & mask_ne
            | self.0[2].alive.down(4).left(4) & mask_sw
            | self.0[3].alive.down(4).right(4) & mask_se;

        Leaf::new(center)
    }
}
