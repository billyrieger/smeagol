// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    bool8x8::Bool8x8,
    grid::{Grid2x2, Grid3x3, Grid4x4},
    Rule,
};
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(u8);

new_key_type! {
    pub struct NodeId;
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

impl Default for Node {
    fn default() -> Self {
        Self::Leaf(Leaf::default())
    }
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
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Leaf {
    alive: Bool8x8,
}

impl Leaf {
    /// # Examples
    ///
    /// ```
    /// # use smeagol::{bool8x8::Bool8x8, node::Leaf};
    /// let all_dead = Leaf::new(Bool8x8::FALSE);
    /// let all_alive = Leaf::new(Bool8x8::TRUE);
    ///
    /// let glider = Leaf::new(Bool8x8(0x0000_1008_3800_0000));
    /// ```
    pub const fn new(alive: Bool8x8) -> Self {
        Self { alive }
    }

    fn _step(&self, rule: Rule) -> Self {
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

        let result = Bool8x8::FALSE
            | dead & alive_neighbor_count[0] & rule.birth[0]
            | dead & alive_neighbor_count[1] & rule.birth[1]
            | dead & alive_neighbor_count[2] & rule.birth[2]
            | dead & alive_neighbor_count[3] & rule.birth[3]
            | dead & alive_neighbor_count[4] & rule.birth[4]
            | dead & alive_neighbor_count[5] & rule.birth[5]
            | dead & alive_neighbor_count[6] & rule.birth[6]
            | dead & alive_neighbor_count[7] & rule.birth[7]
            | dead & alive_neighbor_count[8] & rule.birth[8]
            | alive & alive_neighbor_count[0] & rule.survival[0]
            | alive & alive_neighbor_count[1] & rule.survival[1]
            | alive & alive_neighbor_count[2] & rule.survival[2]
            | alive & alive_neighbor_count[3] & rule.survival[3]
            | alive & alive_neighbor_count[4] & rule.survival[4]
            | alive & alive_neighbor_count[5] & rule.survival[5]
            | alive & alive_neighbor_count[6] & rule.survival[6]
            | alive & alive_neighbor_count[7] & rule.survival[7]
            | alive & alive_neighbor_count[8] & rule.survival[8];

        Self::new(result)
    }
}

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub struct Branch {
    children: Grid2x2<NodeId>,
    level: Level,
    population: u128,
}

#[derive(Clone)]
pub struct Quadtree {
    rule: Rule,
    id_lookup: HashMap<Node, NodeId>,
    nodes: SlotMap<NodeId, Node>,
    steps: SecondaryMap<NodeId, NodeId>,
    jumps: SecondaryMap<NodeId, NodeId>,
}

impl Quadtree {
    pub fn make_leaf(&mut self, leaf: Leaf) -> Option<NodeId> {
        Some(self.get_id(Node::Leaf(leaf)))
    }

    pub fn make_inner(&mut self, children: Grid2x2<NodeId>) -> Option<NodeId> {
        let nodes: Grid2x2<_> = children.try_map(|id| self.get_node(id))?;
        if let [a, b, c, d] = nodes.unpack() {
            let level = a.level();
            debug_assert_eq!(level, b.level());
            debug_assert_eq!(level, c.level());
            debug_assert_eq!(level, d.level());
            let population = a.population() + b.population() + c.population() + d.population();
            let node = Node::Branch(Branch {
                children,
                level,
                population,
            });
            Some(self.get_id(node))
        } else {
            None
        }
    }

    pub fn get_node(&self, id: NodeId) -> Option<Node> {
        self.nodes.get(id).copied()
    }

    pub fn recurse<F, G>(&mut self, children: Grid2x2<NodeId>, f: F, g: G) -> Option<NodeId>
    where
        F: Fn(&mut Self, Grid2x2<NodeId>) -> Option<NodeId>,
        G: Fn(&mut Self, Grid2x2<NodeId>) -> Option<NodeId>,
    {
        let nodes: Grid2x2<_> = children.try_map(|id| self.get_node(id))?;
        match nodes.unpack() {
            &[Node::Leaf(w), Node::Leaf(x), Node::Leaf(y), Node::Leaf(z)] => {
                let grid2x2 = Grid2x2::pack(&[w, x, y, z]);
                self.make_leaf(grid2x2.jump(self.rule))
            }

            &[Node::Branch(w), Node::Branch(x), Node::Branch(y), Node::Branch(z)] => {
                let _: Grid2x2<_> = Grid2x2::pack(&[w, x, y, z]).map(|branch| branch.children);
                // let grid4x4 = Grid4x4::flatten(grandchildren);
                let grid4x4 = Grid4x4::default();
                let partial: Grid3x3<_> = grid4x4.reduce(|x| f(self, x))?;
                let grid2x2: Grid2x2<_> = partial.reduce(|x| g(self, x))?;
                self.make_inner(grid2x2)
            }

            _ => None,
        }
    }

    pub fn jump(&mut self, children: Grid2x2<NodeId>) -> Option<NodeId> {
        self.recurse(children, Self::jump, Self::jump)
        // let nodes: Grid2x2<_> = children.try_map(|id| self.get_node(id))?;
        // match nodes.unpack() {
        //     [Node::Leaf(w), Node::Leaf(x), Node::Leaf(y), Node::Leaf(z)] => {
        //         let grid2x2 = Grid2x2::pack(&[*w, *x, *y, *z]);
        //         self.make_leaf(grid2x2.jump(self.rule))
        //     }

        //     [Node::Branch(w), Node::Branch(x), Node::Branch(y), Node::Branch(z)] => {
        //         let _: Grid2x2<_> = Grid2x2::pack(&[*w, *x, *y, *z]).map(|branch| branch.children);
        //         // let grid4x4 = Grid4x4::flatten(grandchildren);
        //         let grid4x4 = Grid4x4::default();
        //         let partial: Grid3x3<_> = grid4x4.reduce(|x| self.jump(x))?;
        //         let grid2x2: Grid2x2<_> = partial.reduce(|x| self.jump(x))?;
        //         self.make_inner(grid2x2)
        //     }

        //     _ => None,
        // }
    }

    fn get_id(&mut self, node: Node) -> NodeId {
        self.id_lookup.get(&node).copied().unwrap_or_else(|| {
            let id = self.nodes.insert(node);
            self.id_lookup.insert(node, id);
            id
        })
    }
}
