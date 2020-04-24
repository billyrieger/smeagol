// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::leaf::{Bool8x8, Leaf, Rule};
use either::{Either, Left, Right};
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Node {
    kind: NodeKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum NodeKind {
    Leaf(Leaf),
    Inner(Inner),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Inner {
    children: Macrocell<NodeId>,
    level: Level,
    population: u128,
}

impl Node {
    fn from_leaf(leaf: Leaf) -> Self {
        Self {
            kind: NodeKind::Leaf(leaf),
        }
    }

    fn from_inner(inner: Inner) -> Self {
        Self {
            kind: NodeKind::Inner(inner),
        }
    }

    pub fn children(&self) -> Option<Macrocell<NodeId>> {
        match self.kind {
            NodeKind::Leaf(_) => None,
            NodeKind::Inner(inner) => Some(inner.children),
        }
    }

    pub fn level(&self) -> Level {
        match self.kind {
            NodeKind::Leaf(_) => Level(3),
            NodeKind::Inner(inner) => inner.level,
        }
    }

    pub fn population(&self) -> u128 {
        match self.kind {
            NodeKind::Leaf(leaf) => leaf.population(),
            NodeKind::Inner(inner) => inner.population,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(u8);

pub const MAX_LEVEL: Level = Level(64);

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
    pub fn make_id(&mut self, node: Node) -> NodeId {
        self.ids.get(&node).copied().unwrap_or_else(|| {
            let id = self.nodes.insert(node);
            self.ids.insert(node, id);
            id
        })
    }

    pub fn make_node(&mut self, children: Macrocell<NodeId>) -> Option<NodeId> {
        let nodes = children.try_map(|id| self.get_node(id))?;
        let level = nodes.nw.level();
        assert_eq!(level, nodes.ne.level());
        assert_eq!(level, nodes.sw.level());
        assert_eq!(level, nodes.se.level());
        let population = 0
            + nodes.nw.population()
            + nodes.ne.population()
            + nodes.sw.population()
            + nodes.se.population();
        let inner = Inner {
            children,
            level,
            population,
        };
        Some(self.make_id(Node::from_inner(inner)))
    }

    pub fn get_node(&self, id: NodeId) -> Option<Node> {
        self.nodes.get(id).copied()
    }

    fn jump_children(&mut self, children: Macrocell<NodeId>) -> Option<NodeId> {
        match children.try_map(|id| self.get_node(id))?.kind()? {
            Left(macro_leaf) => Some(self.make_id(Node::from_leaf(macro_leaf.jump(self.rule)))),
            Right(macro2_id) => {
                let result = macro2_id
                    .reduce(|x| self.jump_children(x))?
                    .reduce(|x| self.jump_children(x))?;
                self.make_node(result)
            }
        }
    }
}

impl Macrocell<Node> {
    fn kind(self) -> Option<Either<Macrocell<Leaf>, Macrocell2<NodeId>>> {
        match (self.nw.kind, self.ne.kind, self.sw.kind, self.se.kind) {
            (NodeKind::Leaf(nw), NodeKind::Leaf(ne), NodeKind::Leaf(sw), NodeKind::Leaf(se)) => {
                Some(Left(Macrocell { nw, ne, sw, se }))
            }
            (
                NodeKind::Inner(nw),
                NodeKind::Inner(ne),
                NodeKind::Inner(sw),
                NodeKind::Inner(se),
            ) => Some(Right(Macrocell {
                nw: nw.children,
                ne: ne.children,
                sw: sw.children,
                se: se.children,
            })),
            _ => None,
        }
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

pub type Macrocell2<T> = Macrocell<Macrocell<T>>;
pub type Macrocell3<T> = Macrocell<Macrocell2<T>>;

impl<T> Macrocell<T> {
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

struct Partial<T> {
    grid: [[T; 3]; 3],
}

impl<T> Partial<T> {
    fn reduce<F>(&self, mut func: F) -> Option<Macrocell<T>>
    where
        F: FnMut(Macrocell<T>) -> Option<T>,
        T: Copy,
    {
        let w = Macrocell {
            nw: self.grid[0][0],
            ne: self.grid[0][1],
            sw: self.grid[1][0],
            se: self.grid[1][1],
        };
        let w = func(w)?;

        let x = Macrocell {
            nw: self.grid[0][1],
            ne: self.grid[0][2],
            sw: self.grid[1][1],
            se: self.grid[1][2],
        };
        let x = func(x)?;

        let y = Macrocell {
            nw: self.grid[1][0],
            ne: self.grid[1][1],
            sw: self.grid[2][0],
            se: self.grid[2][1],
        };
        let y = func(y)?;

        let z = Macrocell {
            nw: self.grid[1][1],
            ne: self.grid[1][2],
            sw: self.grid[2][1],
            se: self.grid[2][2],
        };
        let z = func(z)?;

        Some(Macrocell {
            nw: w,
            ne: x,
            sw: y,
            se: z,
        })
    }
}

impl<T> Macrocell2<T> {
    fn reduce<F>(&self, mut func: F) -> Option<Partial<T>>
    where
        F: FnMut(Macrocell<T>) -> Option<T>,
        T: Copy,
    {
        let a = Macrocell {
            nw: self.nw.nw,
            ne: self.nw.ne,
            sw: self.nw.sw,
            se: self.nw.se,
        };
        let a = func(a)?;

        let b = Macrocell {
            nw: self.nw.ne,
            ne: self.ne.nw,
            sw: self.nw.se,
            se: self.ne.sw,
        };
        let b = func(b)?;

        let c = Macrocell {
            nw: self.ne.nw,
            ne: self.ne.ne,
            sw: self.ne.sw,
            se: self.ne.se,
        };
        let c = func(c)?;

        let d = Macrocell {
            nw: self.nw.sw,
            ne: self.nw.se,
            sw: self.sw.nw,
            se: self.sw.ne,
        };
        let d = func(d)?;

        let e = Macrocell {
            nw: self.nw.se,
            ne: self.ne.sw,
            sw: self.sw.ne,
            se: self.se.nw,
        };
        let e = func(e)?;

        let f = Macrocell {
            nw: self.ne.sw,
            ne: self.ne.se,
            sw: self.se.nw,
            se: self.se.ne,
        };
        let f = func(f)?;

        let g = Macrocell {
            nw: self.sw.nw,
            ne: self.sw.ne,
            sw: self.sw.sw,
            se: self.sw.se,
        };
        let g = func(g)?;

        let h = Macrocell {
            nw: self.sw.ne,
            ne: self.se.nw,
            sw: self.sw.se,
            se: self.se.sw,
        };
        let h = func(h)?;

        let i = Macrocell {
            nw: self.se.nw,
            ne: self.se.ne,
            sw: self.se.sw,
            se: self.se.se,
        };
        let i = func(i)?;

        Some(Partial {
            grid: [[a, b, c], [d, e, f], [g, h, i]],
        })
    }
}

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
// ┠ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ┃ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ┨
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
// ┠ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ┃ ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ + ╌ ╌ ╌ ╌ ┨
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
