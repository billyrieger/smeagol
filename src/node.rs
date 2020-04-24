// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::leaf::{Bool8x8, Leaf, Rule};
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Node {
    Leaf(Leaf),
    Interior {
        children: Macrocell<NodeId>,
        level: Level,
    },
}

impl Node {
    pub fn level(&self) -> Level {
        match self {
            Node::Leaf(_) => Level(3),
            Node::Interior { level, .. } => *level,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(u8);

new_key_type! {
    pub struct NodeId;
}

#[derive(Clone, Debug)]
pub struct NodeStore {
    rule: Rule,
    lookup: HashMap<Node, NodeId>,
    nodes: SlotMap<NodeId, Node>,
    steps: SecondaryMap<NodeId, Node>,
    jumps: SecondaryMap<NodeId, Node>,
}

impl NodeStore {
    pub fn make_id(&mut self, node: Node) -> NodeId {
        self.lookup.get(&node).copied().unwrap_or_else(|| {
            let id = self.nodes.insert(node);
            self.lookup.insert(node, id);
            id
        })
    }

    pub fn children(&self, id: NodeId) -> Option<Macrocell<NodeId>> {
        match self.get_node(id)? {
            Node::Leaf(_) => None,
            Node::Interior { children, .. } => Some(children),
        }
    }

    pub fn get_node(&self, id: NodeId) -> Option<Node> {
        self.nodes.get(id).copied()
    }

    fn step(&mut self, cells: Macrocell<NodeId>) -> Option<NodeId> {
        todo!()
    }

    fn jump(&mut self, id: NodeId) -> Option<NodeId> {
        todo!()
    }
}

/// A macrocell.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Macrocell<T> {
    /// The northwest child.
    pub nw: T,
    /// The northeast child.
    pub ne: T,
    /// The southwest child.
    pub sw: T,
    /// The southeast child.
    pub se: T,
}

impl<T> Macrocell<T> {
    fn map<F, U>(self, f: F) -> Macrocell<U>
    where
        F: Fn(T) -> U,
    {
        Macrocell {
            nw: f(self.nw),
            ne: f(self.ne),
            sw: f(self.sw),
            se: f(self.se),
        }
    }

    fn try_map<F, U>(self, f: F) -> Option<Macrocell<U>>
    where
        F: Fn(T) -> Option<U>,
    {
        Some(Macrocell {
            nw: f(self.nw)?,
            ne: f(self.ne)?,
            sw: f(self.sw)?,
            se: f(self.se)?,
        })
    }
}

pub type Macrocell2<T> = Macrocell<Macrocell<T>>;
pub type Macrocell3<T> = Macrocell<Macrocell2<T>>;

impl Macrocell<Leaf> {
    pub fn jump(&self, rule: Rule) -> Leaf {
        let a = self.nw.jump(rule);
        let b = self.north().jump(rule);
        let c = self.ne.jump(rule);
        let d = self.west().jump(rule);
        let e = self.center().jump(rule);
        let f = self.east().jump(rule);
        let g = self.sw.jump(rule);
        let h = self.south().jump(rule);
        let i = self.se.jump(rule);

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
        Self::join_horizontal(self.nw, self.ne)
    }

    pub fn south(&self) -> Leaf {
        Self::join_horizontal(self.sw, self.se)
    }

    pub fn east(&self) -> Leaf {
        Self::join_vertical(self.ne, self.se)
    }

    pub fn west(&self) -> Leaf {
        Self::join_vertical(self.nw, self.sw)
    }

    fn center(&self) -> Leaf {
        let mask_nw = Bool8x8(0xF0F0_F0F0_0000_0000);
        let mask_ne = Bool8x8(0x0F0F_0F0F_0000_0000);
        let mask_sw = Bool8x8(0x0000_0000_F0F0_F0F0);
        let mask_se = Bool8x8(0x0000_0000_0F0F_0F0F);

        let center = Bool8x8::FALSE
            | self.nw.alive.up(4).left(4) & mask_nw
            | self.ne.alive.up(4).right(4) & mask_ne
            | self.sw.alive.down(4).left(4) & mask_sw
            | self.se.alive.down(4).right(4) & mask_se;

        Leaf::new(center)
    }
}

// ```text
// ┏━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┳━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┓
// ┃  NW     ╎  NW     ╎  NW     ╎  NW     ┃  NE     ╎  NE     ╎  NE     ╎  NE     ┃
// ┠   nw    ╎   nw    ╎   ne    ╎   ne    ┃   nw    ╎   nw    ╎   ne    ╎   ne    ┨
// ┃    ⁿʷ   ╎    ⁿᵉ   ╎    ⁿʷ   ╎    ⁿᵉ   ┃    ⁿʷ   ╎    ⁿᵉ   ╎    ⁿʷ   ╎    ⁿᵉ   ┃
// ┠ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ╎ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ┃ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ╎ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ┨
// ┃  NW     ╎  NW     ╎  NW     ╎  NW     ┃  NE     ╎  NE     ╎  NE     ╎  NE     ┃
// ┠   nw    ╎   nw    ╎   ne    ╎   ne    ┃   nw    ╎   nw    ╎   ne    ╎   ne    ┨
// ┃    ˢʷ   ╎    ˢᵉ   ╎    ˢʷ   ╎    ˢᵉ   ┃    ˢʷ   ╎    ˢᵉ   ╎    ˢʷ   ╎    ˢᵉ   ┃
// ┠ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ┃ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ┨
// ┃  NW     ╎  NW     ╎  NW     ╎  NW     ┃  NE     ╎  NE     ╎  NE     ╎  NE     ┃
// ┠   sw    ╎   sw    ╎   se    ╎   se    ┃   sw    ╎   sw    ╎   se    ╎   se    ┨
// ┃    ⁿʷ   ╎    ⁿᵉ   ╎    ⁿʷ   ╎    ⁿᵉ   ┃    ⁿʷ   ╎    ⁿᵉ   ╎    ⁿʷ   ╎    ⁿᵉ   ┃
// ┠ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ╎ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ┃ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ╎ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ┨
// ┃  NW     ╎  NW     ╎  NW     ╎  NW     ┃  NE     ╎  NE     ╎  NE     ╎  NE     ┃
// ┠   sw    ╎   sw    ╎   se    ╎   se    ┃   sw    ╎   sw    ╎   se    ╎   se    ┨
// ┃    ˢʷ   ╎    ˢᵉ   ╎    ˢʷ   ╎    ˢᵉ   ┃    ˢʷ   ╎    ˢᵉ   ╎    ˢʷ   ╎    ˢᵉ   ┃
// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
// ┃  SW     ╎  SW     ╎  SW     ╎  SW     ┃  SE     ╎  SE     ╎  SE     ╎  SE     ┃
// ┠   nw    ╎   nw    ╎   ne    ╎   ne    ┃   nw    ╎   nw    ╎   ne    ╎   ne    ┨
// ┃    ⁿʷ   ╎    ⁿᵉ   ╎    ⁿʷ   ╎    ⁿᵉ   ┃    ⁿʷ   ╎    ⁿᵉ   ╎    ⁿʷ   ╎    ⁿᵉ   ┃
// ┠ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ╎ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ┃ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ╎ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ┨
// ┃  SW     ╎  SW     ╎  SW     ╎  SW     ┃  SE     ╎  SE     ╎  SE     ╎  SE     ┃
// ┠   nw    ╎   nw    ╎   ne    ╎   ne    ┃   nw    ╎   nw    ╎   ne    ╎   ne    ┨
// ┃    ˢʷ   ╎    ˢᵉ   ╎    ˢʷ   ╎    ˢᵉ   ┃    ˢʷ   ╎    ˢᵉ   ╎    ˢʷ   ╎    ˢᵉ   ┃
// ┠ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ┃ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ┨
// ┃  SW     ╎  SW     ╎  SW     ╎  SW     ┃  SE     ╎  SE     ╎  SE     ╎  SE     ┃
// ┠   sw    ╎   sw    ╎   se    ╎   se    ┃   sw    ╎   sw    ╎   se    ╎   se    ┨
// ┃    ⁿʷ   ╎    ⁿᵉ   ╎    ⁿʷ   ╎    ⁿᵉ   ┃    ⁿʷ   ╎    ⁿᵉ   ╎    ⁿʷ   ╎    ⁿᵉ   ┃
// ┠ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ╎ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ┃ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ╎ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ┨
// ┃  SW     ╎  SW     ╎  SW     ╎  SW     ┃  SE     ╎  SE     ╎  SE     ╎  SE     ┃
// ┠   sw    ╎   sw    ╎   se    ╎   se    ┃   sw    ╎   sw    ╎   se    ╎   se    ┨
// ┃    ˢʷ   ╎    ˢᵉ   ╎    ˢʷ   ╎    ˢᵉ   ┃    ˢʷ   ╎    ˢᵉ   ╎    ˢʷ   ╎    ˢᵉ   ┃
// ┗━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┻━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┛
// ```

#[cfg(test)]
mod tests {}
