// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::leaf::{Bool8x8, Leaf, Rule};
use either::{Either, Left, Right};
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use std::collections::HashMap;

/// A 2 by 2 grid of values.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid2x2<T>([[T; 2]; 2]);

impl<T> Grid2x2<T> {
    pub fn unpack(self) -> [[T; 2]; 2] {
        self.0
    }
}

/// A 3 by 3 grid of values.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid3x3<T>([[T; 3]; 3]);

impl<T> Grid3x3<T> {
    pub fn unpack(self) -> [[T; 3]; 3] {
        self.0
    }
}

/// A 4 by 4 grid of values.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid4x4<T>([[T; 4]; 4]);

impl<T> Grid4x4<T> {
    pub fn unpack(self) -> [[T; 4]; 4] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Branch {
    children: Grid2x2<NodeId>,
    level: Level,
    population: u128,
}

impl Node {
    pub fn children(&self) -> Option<Grid2x2<NodeId>> {
        match self {
            Self::Leaf(_) => None,
            Self::Branch(branch) => Some(branch.children),
        }
    }

    pub fn level(&self) -> Level {
        match self {
            Self::Leaf(_) => Level(3),
            Self::Branch(branch) => branch.level,
        }
    }

    pub fn population(&self) -> u128 {
        match self {
            Self::Leaf(leaf) => leaf.population(),
            Self::Branch(branch) => branch.population,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(u8);

new_key_type! {
    pub struct NodeId;
}

#[derive(Clone, Debug)]
pub struct Store {
    rule: Rule,
    ids: HashMap<Node, NodeId>,
    nodes: SlotMap<NodeId, Node>,
    steps: SecondaryMap<NodeId, NodeId>,
    jumps: SecondaryMap<NodeId, NodeId>,
}

impl Store {
    fn get_id(&mut self, node: Node) -> NodeId {
        self.ids.get(&node).copied().unwrap_or_else(|| {
            let id = self.nodes.insert(node);
            self.ids.insert(node, id);
            id
        })
    }

    pub fn make_leaf(&mut self, leaf: Leaf) -> Option<NodeId> {
        Some(self.get_id(Node::Leaf(leaf)))
    }

    pub fn make_inner(&mut self, children: Grid2x2<NodeId>) -> Option<NodeId> {
        let [[a, b], [c, d]] = children.try_map(|id| self.get_node(id))?.unpack();
        let level = a.level();
        assert_eq!(level, b.level());
        assert_eq!(level, c.level());
        assert_eq!(level, d.level());
        let population = a.population() + b.population() + c.population() + d.population();
        let branch = Branch {
            children,
            level,
            population,
        };
        Some(self.get_id(Node::Branch(branch)))
    }

    pub fn get_node(&self, id: NodeId) -> Option<Node> {
        self.nodes.get(id).copied()
    }

    pub fn jump(&mut self, children: Grid2x2<NodeId>) -> Option<NodeId> {
        match children.try_map(|id| self.get_node(id))?.unpack() {
            [[Node::Leaf(w), Node::Leaf(x)], [Node::Leaf(y), Node::Leaf(z)]] => {
                let grid2x2 = Grid2x2([[w, x], [y, z]]);
                self.make_leaf(grid2x2.jump(self.rule))
            }

            [[Node::Branch(w), Node::Branch(x)], [Node::Branch(y), Node::Branch(z)]] => {
                let [[a, b], [e, f]] = w.children.unpack();
                let [[c, d], [g, h]] = x.children.unpack();
                let [[i, j], [m, n]] = y.children.unpack();
                let [[k, l], [o, p]] = z.children.unpack();
                let grid4x4 = Grid4x4([[a, b, c, d], [e, f, g, h], [i, j, k, l], [m, n, o, p]]);

                let grid3x3 = grid4x4.reduce(|x| self.jump(x))?;
                let grid2x2 = grid3x3.reduce(|x| self.jump(x))?;

                self.make_inner(grid2x2)
            }

            _ => None,
        }
    }
}

impl<T> Grid2x2<T> {
    pub fn map<F, U>(self, f: F) -> Grid2x2<U>
    where
        F: Fn(T) -> U,
        T: Copy,
    {
        let [[a, b], [c, d]] = self.unpack();
        Grid2x2([[f(a), f(b)], [f(c), f(d)]])
    }

    pub fn try_map<F, U>(self, f: F) -> Option<Grid2x2<U>>
    where
        F: Fn(T) -> Option<U>,
        T: Copy,
    {
        let [[a, b], [c, d]] = self.unpack();
        Some(Grid2x2([[f(a)?, f(b)?], [f(c)?, f(d)?]]))
    }
}

impl<T> Grid3x3<T> {
    fn reduce<F>(&self, mut kernel: F) -> Option<Grid2x2<T>>
    where
        F: FnMut(Grid2x2<T>) -> Option<T>,
        T: Copy,
    {
        let [[a, b, c], [d, e, f], [g, h, i]] = self.unpack();

        let w = kernel(Grid2x2([[a, b], [d, e]]))?;
        let x = kernel(Grid2x2([[b, c], [e, f]]))?;
        let y = kernel(Grid2x2([[d, e], [g, h]]))?;
        let z = kernel(Grid2x2([[e, f], [h, i]]))?;

        Some(Grid2x2([[w, x], [y, z]]))
    }
}

impl<T> Grid4x4<T> {
    fn reduce<F>(&self, mut kernel: F) -> Option<Grid3x3<T>>
    where
        F: FnMut(Grid2x2<T>) -> Option<T>,
        T: Copy,
    {
        let [[a, b, c, d], [e, f, g, h], [i, j, k, l], [m, n, o, p]] = self.unpack();

        let r = kernel(Grid2x2([[a, b], [e, f]]))?;
        let s = kernel(Grid2x2([[b, c], [f, g]]))?;
        let t = kernel(Grid2x2([[c, d], [g, h]]))?;
        let u = kernel(Grid2x2([[e, f], [i, j]]))?;
        let v = kernel(Grid2x2([[f, g], [j, k]]))?;
        let w = kernel(Grid2x2([[g, h], [k, l]]))?;
        let x = kernel(Grid2x2([[i, j], [m, n]]))?;
        let y = kernel(Grid2x2([[j, k], [n, o]]))?;
        let z = kernel(Grid2x2([[k, l], [o, p]]))?;

        Some(Grid3x3([[r, s, t], [u, v, w], [x, y, z]]))
    }
}

impl Grid2x2<Node> {
    fn kind(self) -> Either<Grid2x2<Leaf>, Grid4x4<NodeId>> {
        let [[a, b], [c, d]] = self.unpack();
        match [a, b, c, d] {
            [Node::Leaf(nw), Node::Leaf(ne), Node::Leaf(sw), Node::Leaf(se)] => {
                Left(Grid2x2([[nw, ne], [sw, se]]))
            }

            [Node::Branch(nw), Node::Branch(ne), Node::Branch(sw), Node::Branch(se)] => {
                let [[a, b], [e, f]] = nw.children.unpack();
                let [[c, d], [g, h]] = ne.children.unpack();
                let [[i, j], [m, n]] = sw.children.unpack();
                let [[k, l], [o, p]] = se.children.unpack();
                Right(Grid4x4([
                    [a, b, c, d],
                    [e, f, g, h],
                    [i, j, k, l],
                    [m, n, o, p],
                ]))
            }

            _ => panic!("this shouldn't happen"),
        }
    }
}

impl Grid2x2<Leaf> {
    pub fn jump(&self, rule: Rule) -> Leaf {
        let a = self.0[0][0].jump(rule);
        let b = self.north().jump(rule);
        let c = self.0[0][1].jump(rule);
        let d = self.west().jump(rule);
        let e = self.center().jump(rule);
        let f = self.east().jump(rule);
        let g = self.0[1][0].jump(rule);
        let h = self.south().jump(rule);
        let i = self.0[1][1].jump(rule);

        let mask_center = Bool8x8(0x0000_3C3C_3C3C_0000);
        let combine_jumps = |nw: Leaf, ne: Leaf, sw: Leaf, se: Leaf| {
            Leaf::new(
                Bool8x8::FALSE
                    | (nw.alive & mask_center).up(2).left(2)
                    | ne.mask(mask_center).alive.up(2).right(2)
                    | sw.mask(mask_center).alive.down(2).left(2)
                    | se.mask(mask_center).alive.down(2).right(2),
            )
        };

        let w = combine_jumps(a, b, d, e).jump(rule);
        let x = combine_jumps(b, c, e, f).jump(rule);
        let y = combine_jumps(d, e, g, h).jump(rule);
        let z = combine_jumps(e, f, h, i).jump(rule);

        combine_jumps(w, x, y, z)
    }

    fn join_horizontal(left: Leaf, right: Leaf) -> Leaf {
        let mask_left = Bool8x8(0xFF00_FF00_FF00_FF00);
        let mask_right = Bool8x8(0x00FF00_00FF_00FF_00FF);
        Leaf::new((left.alive.left(4) & mask_left) | (right.alive.right(4) & mask_right))
    }

    fn join_vertical(top: Leaf, bottom: Leaf) -> Leaf {
        let mask_top = Bool8x8(0xFFFF_FFFF_0000_0000);
        let mask_bottom = Bool8x8(0x0000_0000_FFFF_FFFF);
        Leaf::new((top.alive.up(4) & mask_top) | (bottom.alive.down(4) & mask_bottom))
    }

    pub fn north(&self) -> Leaf {
        Self::join_horizontal(self.0[0][0], self.0[0][1])
    }

    pub fn south(&self) -> Leaf {
        Self::join_horizontal(self.0[1][0], self.0[1][1])
    }

    pub fn east(&self) -> Leaf {
        Self::join_vertical(self.0[0][0], self.0[1][0])
    }

    pub fn west(&self) -> Leaf {
        Self::join_vertical(self.0[0][1], self.0[1][1])
    }

    fn center(&self) -> Leaf {
        let mask_nw = Bool8x8(0xF0F0_F0F0_0000_0000);
        let mask_ne = Bool8x8(0x0F0F_0F0F_0000_0000);
        let mask_sw = Bool8x8(0x0000_0000_F0F0_F0F0);
        let mask_se = Bool8x8(0x0000_0000_0F0F_0F0F);

        let center = Bool8x8::FALSE
            | self.0[0][0].alive.up(4).left(4) & mask_nw
            | self.0[0][1].alive.up(4).right(4) & mask_ne
            | self.0[1][0].alive.down(4).left(4) & mask_sw
            | self.0[1][1].alive.down(4).right(4) & mask_se;

        Leaf::new(center)
    }
}

#[cfg(test)]
mod tests {}
